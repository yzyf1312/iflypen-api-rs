//! # IFlyPen API Rust SDK
//!
//! 基于对讯飞听见客户端协议的逆向分析实现，用于访问 IFlyPen API。
//! 本SDK允许用户将本地音频文件快速转换为文字，无需使用讯飞录音笔硬件。

// 导出公共模块
pub mod api;
pub mod error;
pub(crate) mod util;

// 重新导出常用类型，方便用户直接使用
pub use api::{IflyrecClient, TranscriptionOptions, TranscriptionOrder, TranscriptResult};
pub use error::IflyrecError;
