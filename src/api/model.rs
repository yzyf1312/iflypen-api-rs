use serde::{Deserialize, Serialize};

/// 文件上传API响应
#[derive(Debug, Deserialize)]
pub struct FileUploadApiResponse {
    pub code: String,
    pub desc: String,
    pub biz: FileUploadApiBizData,
}

/// 文件上传API业务数据
#[derive(Debug, Deserialize)]
pub struct FileUploadApiBizData {
    #[serde(rename = "fileId")]
    pub file_id: String,
}

/// 提交转录任务响应
#[derive(Debug, Deserialize)]
pub struct SubmitTranscriptionOrderResponse {
    pub code: String,
    pub desc: String,
    pub biz: Option<BizData>,
}

/// 业务数据
#[derive(Debug, Deserialize)]
pub struct BizData {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub success: bool,
}

/// 转录选项
#[derive(Clone, Debug)]
pub struct TranscriptionOptions {
    pub need_sms: bool,
    /// 热词，多个词语之间请使用中文逗号“，”分隔。
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

/// 音频元数据
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

/// 获取订单结果响应
#[derive(Debug, Deserialize)]
pub struct GetOrderResultResponse {
    // pub code: String,
    // pub desc: String,
    pub biz: GetOrderResultBiz,
}

/// 获取订单结果业务数据
#[derive(Debug, Deserialize)]
pub struct GetOrderResultBiz {
    // #[serde(rename = "type")]
    // pub type_num: i32,
    #[serde(rename = "transcriptResult")]
    pub transcript_result: String,
    // #[serde(rename = "saveTime")]
    // pub save_time: i64,
    // pub version: i64,
    // #[serde(rename = "hjFrom")]
    // pub hj_from: i32,
    // #[serde(rename = "languageType")]
    // pub language_type: i32,
}

/// 转录结果
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

/// 段落
#[derive(Debug, Deserialize)]
pub struct Paragraph {
    // paragraph_time 应该为一个 2 元素的数组，分别表示开始和结束时间
    #[serde(rename = "pTime")]
    pub paragraph_time: Vec<i64>,
    pub role: String,
    pub words: Vec<Word>,
}

/// 单词
#[derive(Debug, Deserialize)]
pub struct Word {
    pub modal: bool,
    #[serde(rename = "rl")]
    pub role: String,
    pub text: String,
    pub time: Vec<i64>,
    pub wp: String,
}

/// 角色
#[derive(Debug, Deserialize)]
pub struct Role {
    pub name: String,
    pub role: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// 获取最近订单响应
#[derive(Debug, Deserialize)]
pub struct GetRecentOrdersResponse {
    pub biz: GetRecentOrdersBiz,
    // pub code: String,
    // pub desc: String,
}

/// 获取最近订单业务数据
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecentOrdersBiz {
    // pub count: i32,
    pub hj_list: Vec<TranscriptionOrder>,
    // pub scroll_up_query_param: ScrollQueryParam,
    // pub scroll_down_query_param: ScrollQueryParam,
}

/// 转录订单
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

/// 滚动查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollQueryParam {
    // pub hj_id: Option<String>,
    // pub transcript_id: String,
    // pub sort_hj_create_time: Option<i64>,
    // pub sort_trans_create_time: i64,
}
