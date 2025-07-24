use crate::api::constants::*;
use crate::api::model::*;
use crate::error::{map_api_error, IflyrecError};
use crate::util::{calculate_wav_duration, extract_task_name, generate_random_file_name};

use reqwest::{Client, Method, RequestBuilder};
use secrecy::{ExposeSecret, Secret};
use serde_json::json;
use std::fs;
use std::path::Path;
use tokio::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;

/// 讯飞听见API客户端
pub struct IflyrecClient {
    /// 会话ID（敏感信息，使用Secret包装）
    session_id: Secret<String>,
    /// HTTP客户端
    http_client: Client,
}

impl IflyrecClient {
    /// 创建新的客户端实例
    pub fn new(session_id: String) -> Self {
        Self {
            session_id: Secret::new(session_id),
            http_client: Client::new(),
        }
    }

    /// 构建请求构建器
    ///
    /// 添加通用的头部信息，如Content-Type、X-Biz-Id和X-Session-Id
    fn build_request(&self, method: Method, url: &str, content_type: &str) -> RequestBuilder {
        self.http_client
            .request(method, url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Content-Type", content_type)
            .header("X-Biz-Id", BIZ_ID)
            .header("X-Session-Id", self.session_id.expose_secret())
    }

    /// 提交转录订单
    /// 
    /// 使用文件ID和转录选项创建新的转录任务
    pub async fn submit_transcription_order(
        &self,
        file_id: &str,
        options: Option<TranscriptionOptions>,
    ) -> Result<String, IflyrecError> {
        let opts = options.unwrap_or_default();

        let payload = json!({
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
        });

        let response = self
            .build_request(
                Method::POST,
                SUBMIT_TRANSCRIPTION_ORDER_URL,
                "application/json",
            )
            .json(&payload)
            .send()
            .await?
            .json::<SubmitTranscriptionOrderResponse>()
            .await?;

        if response.code != "000000" {
            return Err(map_api_error(&response.code, &response.desc));
        }

        let order_id = response
            .biz
            .and_then(|biz| {
                if biz.success {
                    Some(biz.order_id)
                } else {
                    None
                }
            })
            .ok_or(IflyrecError::OrderIdUnavailable)?;

        Ok(order_id)
    }

    /// 计算讯飞服务器上的音频时长
    /// 
    /// 在提交转录任务前需要调用此方法
    pub async fn calculate_duration_on_iflyrec(
        &self,
        file_id: &str,
    ) -> Result<(), IflyrecError> {
        let url = CALCULATE_DURATION_URL_TEMPLATE.replacen("{}", file_id, 1);

        let response = self
            .build_request(Method::POST, &url, "application/json")
            .header("Content-Length", "0")
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(IflyrecError::DurationCalculationError(format!(
                "Failed to calculate duration for file_id: {file_id}"
            )))
        }
    }

    /// 获取最近的转录订单列表
    pub async fn get_recent_orders(&self) -> Result<Vec<TranscriptionOrder>, IflyrecError> {
        let response = self
            .build_request(Method::POST, GET_RECENT_ORDERS_URL, "application/json")
            .body("{}")
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_data = serde_json::from_str::<GetRecentOrdersResponse>(&response_text)?;

        Ok(response_data.biz.hj_list)
    }

    /// 根据订单ID获取特定订单
    pub async fn get_order(
        &self,
        order_id: &str,
    ) -> Result<Option<TranscriptionOrder>, IflyrecError> {
        let orders = self.get_recent_orders().await?;
        let order = orders.into_iter().find(|item| item.order_id == order_id);
        Ok(order)
    }

    /// 上传音频文件
    /// 
    /// 将本地音频文件上传到讯飞服务器
    pub async fn upload_audio_file(
        &self,
        audio_path_str: &str,
        task_name: Option<String>,
    ) -> Result<String, IflyrecError> {
        let audio_path = Path::new(audio_path_str);

        let task_name = extract_task_name(audio_path, task_name);

        // 获取文件元数据和时长
        let metadata = fs::metadata(audio_path)?;
        let file_size = metadata.len();
        let audio_time = calculate_wav_duration(file_size);

        // 创建初始元数据以获取 file_id
        let initial_metadata = AudioMetadata::new(
            task_name.to_string(),
            format!("{}{}", DEFAULT_AUDIO_PATH_PREFIX, generate_random_file_name()),
            file_size,
            audio_time,
            0,
            0,
            String::new(),
        );

        let header_block = initial_metadata.to_binary_block()?;

        // 发送初始请求获取 file_id
        let response = self
            .build_request(Method::POST, FILE_UPLOAD_URL, "application/octet-stream")
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
            .build_request(Method::POST, FILE_UPLOAD_URL, "application/octet-stream")
            .body(final_data)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            let api_response: FileUploadApiResponse = serde_json::from_str(&response_text)?;
            if api_response.code == "000000" && api_response.desc == "success" {
                Ok(file_id)
            } else {
                Err(IflyrecError::UploadError(format!(
                    "{} - {}",
                    api_response.code, api_response.desc
                )))
            }
        } else {
            Err(IflyrecError::UploadError(format!(
                "Upload request failed with status: {}", 
                response.status()
            )))
        }
    }

    /// 初始化转录任务
    /// 
    /// 上传音频文件并提交转录任务
    pub async fn initiate_transcription_task(
        &self,
        audio_path_str: &str,
        task_name: Option<String>,
        options: Option<TranscriptionOptions>,
    ) -> Result<String, IflyrecError> {
        // 调用上传函数获取 file_id
        let file_id = self.upload_audio_file(audio_path_str, task_name).await?;

        // 后续处理步骤
        self.calculate_duration_on_iflyrec(&file_id).await?;

        let retry_strategy = ExponentialBackoff::from_millis(500)
            .max_delay(Duration::from_secs(10))
            .take(5); // 最多重试5次

        let order_id = Retry::spawn(retry_strategy, || async {
            match self
                .submit_transcription_order(&file_id, options.clone())
                .await
            {
                Ok(order_id) => Ok(order_id),
                Err(e) => {
                    // 检查是否是可重试的错误（订单音频时长计算中）
                    if let IflyrecError::ApiError { desc, .. } = &e {
                        if desc.contains("订单音频时长计算中") {
                            // 标记为可重试的错误，但不改变错误本身
                            tracing::info!("订单音频时长计算中，可以重试");
                        }
                    }
                    Err(e)
                }
            }
        })
        .await?;

        Ok(order_id)
    }

    /// 获取订单转录结果
    pub async fn get_order_result(
        &self,
        order: &TranscriptionOrder,
    ) -> Result<TranscriptResult, IflyrecError> {
        let url = GET_ORDER_RESULT_URL_TEMPLATE
            .replacen("{}", &order.order_id, 1)
            .replacen("{}", &order.origin_audio_id, 1);

        let response = self
            .build_request(Method::GET, &url, "application/json")
            .send()
            .await?;

        let response_text = response.text().await?;
        let response_data = serde_json::from_str::<GetOrderResultResponse>(&response_text)?;
        let text = response_data.biz.transcript_result;
        let transcript_result = serde_json::from_str::<TranscriptResult>(&text)?;

        Ok(transcript_result)
    }
}
