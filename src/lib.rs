pub mod api {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::path::Path;
    use tokio::time::Duration;
    use tokio::time::sleep;

    /// 文件上传URL
    pub const FILE_UPLOAD_URL: &str =
        "https://www.iflyrec.com/AudioStreamService/v1/audios?type=block";

    #[derive(Debug, Deserialize)]
    pub struct FileUploadApiResponse {
        pub code: String,
        pub desc: String,
        pub biz: FileUploadApiBizData,
    }

    #[derive(Debug, Deserialize)]
    pub struct FileUploadApiBizData {
        #[serde(rename = "fileId")]
        pub file_id: String,
    }

    /// 接收文件大小（单位：字节），返回毫秒整数时长字符串
    #[inline]
    pub fn calculate_wav_duration(file_size: u64) -> String {
        (file_size / 32).to_string()
    }

    #[derive(Debug, Deserialize)]
    pub struct ApiResponse {
        pub code: String,
        pub desc: String,
        pub biz: Option<BizData>,
    }

    #[derive(Debug, Deserialize)]
    pub struct BizData {
        #[serde(rename = "orderId")]
        pub order_id: String,
        pub success: bool,
    }

    pub struct IflyrecClient {
        pub session_id: String,
        pub http_client: Client,
    }

    impl IflyrecClient {
        pub fn new(session_id: String) -> Self {
            Self {
                session_id: session_id,
                http_client: Client::new(),
            }
        }

        pub async fn submit_transcription_order(
            &self,
            file_id: String,
            session_id: String,
            options: Option<TranscriptionOptions>,
        ) -> Result<String, Box<dyn Error>> {
            let opts = options.unwrap_or_else(|| TranscriptionOptions::new());

            let payload = json!(
                {
                    "professionalField": "",
                    "orderType": 1,
                    "hotWords": opts.hot_words,
                    "needSms": if opts.need_sms { "1" } else { "0" },
                    "files": [{
                        "roleNum": "",
                        "keyWords": "",
                        "hotWords": opts.hot_words,
                        "fileFrom": "client",
                        "fileId": file_id,
                        "audioFrom": "B1"
                    }],
                    "language": opts.language,
                    "orderName": "",
                    "subtitleCount": ""
                }
            );

            let response = self
                .http_client
                .post("https://www.iflyrec.com/XFTJPCAdaptService/v1/B1/orders/")
                .header("Accept", "application/json, text/plain, */*")
                .header("Content-Type", "application/json")
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", session_id)
                .json(&payload)
                .send()
                .await?;

            let api_response: ApiResponse = response.json().await?;
            if api_response.code != "000000" {
                return Err(format!("API error: {}", api_response.desc).into());
            }

            let order_id = api_response
                .biz
                .and_then(|biz| {
                    if biz.success {
                        Some(biz.order_id)
                    } else {
                        None
                    }
                })
                .ok_or("Failed to get order ID")?;

            Ok(order_id)
        }

        pub async fn calculate_duration_on_iflyrec(
            &self,
            file_id: String,
        ) -> Result<(), Box<dyn Error>> {
            let response = self
                .http_client
                .post(format!(
                    "https://www.iflyrec.com/TranscriptOrderService/v1/tempAudios/{}/calculateDuration",
                    file_id
                ))
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", self.session_id.clone())
                .header("Content-Length", "0")
                .send()
                .await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("Failed to calculate duration for file_id: {}", file_id).into())
            }
        }

        pub async fn upload_audio_file(
            &self,
            audio_path_str: &str,
            task_name: Option<String>,
        ) -> Result<String, Box<dyn Error>> {
            let audio_path = Path::new(audio_path_str);

            // 提取文件名和任务名
            let file_name = audio_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

            let task_name: String = match task_name {
                Some(name) => name,
                None => audio_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output")
                    .into(),
            };

            // 获取文件元数据和时长
            let metadata = fs::metadata(audio_path)?;
            let file_size = metadata.len();
            let audio_time = calculate_wav_duration(file_size);

            // 创建初始元数据以获取 file_id
            let initial_metadata = AudioMetadata::new(
                task_name.to_string(),
                format!("tjb1/{}", file_name),
                file_size,
                audio_time,
                0,
                0,
                String::new(),
            );

            let header_block = initial_metadata.to_binary_block()?;

            // 发送初始请求获取 file_id
            let response = self
                .http_client
                .post(FILE_UPLOAD_URL)
                .header("Accept", "application/json, text/plain, */*")
                .header("Content-Type", "application/octet-stream")
                .header("X-Biz-Id", "tjzs")
                .body(header_block)
                .send()
                .await?;

            let response_text = response.text().await?;
            let api_response: FileUploadApiResponse = serde_json::from_str(&response_text)?;
            let file_id = api_response.biz.file_id;

            // 创建最终元数据
            let final_metadata = AudioMetadata {
                is_last: 1,
                block_index: 1,
                file_id: file_id.clone(),
                ..initial_metadata
            };

            let final_header_block = final_metadata.to_binary_block()?;
            let audio_bytes = fs::read(audio_path)?;
            let mut final_data = final_header_block;
            final_data.extend_from_slice(&audio_bytes);

            // 发送上传请求
            let response = self
                .http_client
                .post(FILE_UPLOAD_URL)
                .header("Accept", "application/json, text/plain, */*")
                .header("Content-Type", "application/octet-stream")
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", self.session_id.clone())
                .body(final_data)
                .send()
                .await?;

            if response.status().is_success() {
                let response_text = response.text().await?;
                let api_response: FileUploadApiResponse = serde_json::from_str(&response_text)?;
                if api_response.code == "000000" && api_response.desc == "success" {
                    Ok(file_id)
                } else {
                    Err(format!(
                        "Upload failed: {} - {}",
                        api_response.code, api_response.desc
                    )
                    .into())
                }
            } else {
                Err(format!("Upload request failed with status: {}", response.status()).into())
            }
        }

        pub async fn initiate_transcription_task(
            &self,
            audio_path_str: &str,
            task_name: Option<String>,
        ) -> Result<String, Box<dyn Error>> {
            // 调用上传函数获取 file_id
            let file_id = self.upload_audio_file(audio_path_str, task_name).await?;

            // 后续处理步骤
            self.calculate_duration_on_iflyrec(file_id.clone()).await?;

            let order_id = loop {
                match self
                    .submit_transcription_order(file_id.clone(), self.session_id.clone(), None)
                    .await
                {
                    Ok(order_id) => break order_id,
                    Err(e) => {
                        if e.to_string().contains("订单音频时长计算中") {
                            sleep(Duration::from_millis(500)).await;
                        } else {
                            return Err(e);
                        }
                    }
                }
            };

            Ok(order_id)
        }
    }

    #[derive(Default)]
    pub struct TranscriptionOptions {
        pub need_sms: bool,
        pub hot_words: String,
        pub language: String,
    }

    impl TranscriptionOptions {
        pub fn new() -> Self {
            Self {
                need_sms: false,
                hot_words: String::new(),
                language: "cn".to_string(),
            }
        }
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AudioMetadata {
        pub audio_name: String,
        pub audio_path: String,
        pub audio_size: u64,
        pub is_last: u8,
        pub block_index: u64,
        pub audio_time: String,
        pub file_id: String,
    }

    impl AudioMetadata {
        pub fn new(
            audio_name: String,
            audio_path: String,
            audio_size: u64,
            audio_time: String,
            is_last: u8,
            block_index: u64,
            file_id: String,
        ) -> Self {
            Self {
                audio_name,
                audio_path,
                audio_size,
                is_last,
                block_index,
                audio_time,
                file_id,
            }
        }

        pub fn to_binary_block(&self) -> Result<Vec<u8>, serde_json::Error> {
            let json_payload = serde_json::to_string(self)?;
            let json_bytes = json_payload.as_bytes();
            let len = (json_bytes.len() as u32).to_be_bytes();

            let mut data_block = Vec::with_capacity(4 + json_bytes.len());
            data_block.extend_from_slice(&len);
            data_block.extend_from_slice(json_bytes);

            Ok(data_block)
        }
    }
}
