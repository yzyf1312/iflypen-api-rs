pub mod api {
    use rand::Rng;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::error::Error;
    use std::fs;
    use std::path::Path;
    use tokio::time::Duration;
    use tokio_retry::Retry;
    use tokio_retry::strategy::ExponentialBackoff;

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
    pub struct SubmitTranscriptionOrderResponse {
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
                session_id,
                http_client: Client::new(),
            }
        }

        pub async fn submit_transcription_order(
            &self,
            file_id: String,
            options: Option<TranscriptionOptions>,
        ) -> Result<String, Box<dyn Error>> {
            let opts = options.unwrap_or_default();

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
                .header("X-Session-Id", self.session_id.clone())
                .json(&payload)
                .send()
                .await?;

            let api_response: SubmitTranscriptionOrderResponse = response.json().await?;
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
                    "https://www.iflyrec.com/TranscriptOrderService/v1/tempAudios/{file_id}/calculateDuration"
                ))
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", self.session_id.clone())
                .header("Content-Length", "0")
                .send()
                .await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("Failed to calculate duration for file_id: {file_id}").into())
            }
        }

        pub async fn get_recent_orders(&self) -> Result<Vec<TranscriptionOrder>, Box<dyn Error>> {
            let response = self
                .http_client
                .post(
                    "https://www.iflyrec.com/XFTJWebAdaptService/v2/hjProcess/recentOperationFiles",
                )
                .header("Accept", "application/json, text/plain, */*")
                .header("Content-Type", "application/json")
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", self.session_id.clone())
                .body("{}")
                .send()
                .await;

            let response_data = serde_json::from_str::<GetRecentOrdersResponse>(
                &response.unwrap().text().await.unwrap(),
            )?;

            Ok(response_data.biz.hj_list)
        }

        pub async fn get_order(
            &self,
            order_id: String,
        ) -> Result<Option<TranscriptionOrder>, Box<dyn Error>> {
            let response = self.get_recent_orders().await?;

            let response_data = response
                .iter()
                .find(|item| item.order_id == order_id)
                .cloned();

            Ok(response_data)
        }
        pub async fn upload_audio_file(
            &self,
            audio_path_str: &str,
            task_name: Option<String>,
        ) -> Result<String, Box<dyn Error>> {
            let audio_path = Path::new(audio_path_str);

            // 提取文件名和任务名
            // let file_name = audio_path
            //     .file_name()
            //     .and_then(|s| s.to_str())
            //     .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;

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
                format!("tjb1/{}", generate_random_file_name()),
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
            options: Option<TranscriptionOptions>,
        ) -> Result<String, Box<dyn Error>> {
            // 调用上传函数获取 file_id
            let file_id = self.upload_audio_file(audio_path_str, task_name).await?;

            // 后续处理步骤
            self.calculate_duration_on_iflyrec(file_id.clone()).await?;

            let retry_strategy = ExponentialBackoff::from_millis(500)
                .max_delay(Duration::from_secs(10))
                .take(5); // 最多重试5次

            let order_id = Retry::spawn(retry_strategy, || async {
                match self
                    .submit_transcription_order(file_id.clone(), options.clone())
                    .await
                {
                    Ok(order_id) => Ok(order_id),
                    Err(e) => {
                        if e.to_string().contains("订单音频时长计算中") {
                            Err(e) // 可重试的错误
                        } else {
                            return Err(e); // 不可重试的错误
                        }
                    }
                }
            })
            .await?;

            Ok(order_id)
        }

        pub async fn get_order_result(
            &self,
            order: &TranscriptionOrder,
        ) -> Result<TranscriptResult, Box<dyn Error>> {
            let url = format!(
                "https://www.iflyrec.com/XFTJWebAdaptService/v1/hyjy/{}/transcriptResults/16?fileSource=app&originAudioId={}",
                order.order_id, order.origin_audio_id
            );

            let response = self
                .http_client
                .get(url)
                .header("Accept", "application/json, text/plain, */*")
                .header("X-Biz-Id", "tjzs")
                .header("X-Session-Id", self.session_id.clone())
                .send()
                .await;

            let response_data = serde_json::from_str::<GetOrderResultResponse>(
                &response.unwrap().text().await.unwrap(),
            )?;
            let text = response_data.biz.transcript_result;
            let response_data = serde_json::from_str::<TranscriptResult>(&text)?;

            Ok(response_data)
        }
    }

    #[derive(Clone, Debug)]
    pub struct TranscriptionOptions {
        pub need_sms: bool,
        pub hot_words: String,
        pub language: String,
    }

    impl Default for TranscriptionOptions {
        fn default() -> Self {
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

    #[derive(Debug, Deserialize)]
    pub struct GetOrderResultResponse {
        pub code: String,
        pub desc: String,
        pub biz: GetOrderResultBiz,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetOrderResultBiz {
        #[serde(rename = "type")]
        pub type_num: i32,
        #[serde(rename = "transcriptResult")]
        pub transcript_result: String,
        #[serde(rename = "saveTime")]
        pub save_time: i64,
        pub version: i64,
        #[serde(rename = "hjFrom")]
        pub hj_from: i32,
        #[serde(rename = "languageType")]
        pub language_type: i32,
    }

    #[derive(Debug, Deserialize)]
    pub struct TranscriptResult {
        // pub images: Vec<Value>,
        #[serde(rename = "ps")]
        pub paragraphs: Vec<Paragraph>,
        pub roles: Vec<Role>,
        // #[serde(rename = "sjResult")]
        // pub sj_result: Vec<Value>,
        // pub styles: Vec<Value>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Paragraph {
        // paragraph_time 应该为一个 2 元素的数组，分别表示开始和结束时间
        #[serde(rename = "pTime")]
        pub paragraph_time: Vec<i64>,
        pub role: String,
        pub words: Vec<Word>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Word {
        pub modal: bool,
        #[serde(rename = "rl")]
        pub role: String,
        pub text: String,
        pub time: Vec<i64>,
        pub wp: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Role {
        pub name: String,
        pub role: String,
        #[serde(rename = "updatedAt")]
        pub updated_at: i64,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetRecentOrdersResponse {
        pub biz: GetRecentOrdersBiz,
        pub code: String,
        pub desc: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetRecentOrdersBiz {
        pub count: i32,
        pub hj_list: Vec<TranscriptionOrder>,
        pub scroll_up_query_param: ScrollQueryParam,
        pub scroll_down_query_param: ScrollQueryParam,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TranscriptionOrder {
        pub order_id: String,
        pub origin_audio_id: String,
        pub order_status: String,
        pub order_name: String,
        pub create_time: i64,
        pub favorite_time: Option<i64>,
        pub last_operate_time: i64,
        pub audio_durations: i64,
        pub hj_from: String,
        pub hj_from_desc: String,
        pub keyword: Vec<String>,
        pub full_text_abstract: Option<String>,
        pub hj_size: i64,
        pub favorite_status: Option<String>,
        pub hj_status: Option<String>,
        pub file_source: String,
        pub file_re_source: String,
        pub hj_lock_status: String,
        pub order_type: String,
        pub transcript_status: Option<String>,
        pub type_: i32,
        pub output_type: i32,
        pub expedite_transcript: String,
        pub file_id: String,
        pub red_point_status: i32,
        pub example_order: Option<String>,
        pub thumbnail_link_list: Option<Vec<String>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ScrollQueryParam {
        pub hj_id: Option<String>,
        pub transcript_id: String,
        pub sort_hj_create_time: Option<i64>,
        pub sort_trans_create_time: i64,
    }

    fn generate_random_file_name() -> String {
        let mut rng = rand::rng();
        let random_number: u32 = rng.random_range(1000..=9999);
        format!("audio_{}.wav", random_number)
    }
}
