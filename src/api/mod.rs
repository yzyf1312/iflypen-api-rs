//! API模块
//! 
//! 包含与讯飞听见API交互的所有功能

mod constants;
mod client;
mod model;

// 重新导出公共类型和函数
pub use client::IflyrecClient;
pub use constants::*;
pub use model::{AudioMetadata, TranscriptionOptions, TranscriptionOrder, TranscriptResult};