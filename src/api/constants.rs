//! API 常量定义

/// 文件上传URL
pub const FILE_UPLOAD_URL: &str =
    "https://www.iflyrec.com/AudioStreamService/v1/audios?type=block";

/// 提交转录订单URL
pub const SUBMIT_TRANSCRIPTION_ORDER_URL: &str =
    "https://www.iflyrec.com/XFTJPCAdaptService/v1/B1/orders/";

/// 获取最近订单URL
pub const GET_RECENT_ORDERS_URL: &str =
    "https://www.iflyrec.com/XFTJWebAdaptService/v2/hjProcess/recentOperationFiles";

/// 计算音频时长URL模板
pub const CALCULATE_DURATION_URL_TEMPLATE: &str =
    "https://www.iflyrec.com/TranscriptOrderService/v1/tempAudios/{}/calculateDuration";

/// 获取订单结果URL模板
pub const GET_ORDER_RESULT_URL_TEMPLATE: &str =
    "https://www.iflyrec.com/XFTJWebAdaptService/v1/hyjy/{}/transcriptResults/16?fileSource=app&originAudioId={}";

/// 业务ID
pub const BIZ_ID: &str = "tjzs";

/// 默认语言
pub const DEFAULT_LANGUAGE: &str = "cn";

/// 默认音频路径前缀
pub const DEFAULT_AUDIO_PATH_PREFIX: &str = "tjb1/";

/// 成功响应码
pub const SUCCESS_CODE: &str = "000000";

/// 成功响应描述
pub const SUCCESS_DESC: &str = "success";