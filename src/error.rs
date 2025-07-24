use thiserror::Error;

#[derive(Error, Debug)]
pub enum IflyrecError {
    #[error("API Error: {code} - {desc}")]
    ApiError { code: String, desc: String },

    #[error("HTTP Request Failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON Deserialization Failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Order still processing: {0}")]
    OrderProcessing(String),

    #[error("Failed to get order ID")]
    OrderIdUnavailable,

    #[error("Duration calculation failed: {0}")]
    DurationCalculationError(String),

    #[error("Upload failed: {0}")]
    UploadError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 用于处理API响应中的错误码
pub fn map_api_error(code: &str, desc: &str) -> IflyrecError {
    match code {
        "000000" => IflyrecError::Unknown(format!("Unexpected error mapping for success code: {desc}")),
        _ => IflyrecError::ApiError {
            code: code.to_string(),
            desc: desc.to_string(),
        },
    }
}