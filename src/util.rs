use rand::Rng;
use std::path::Path;

/// 接收文件大小（单位：字节），返回毫秒整数时长字符串
/// 
/// 这是一个简化的计算方法，实际应用中应根据音频格式和采样率进行更精确的计算
#[inline]
pub fn calculate_wav_duration(file_size: u64) -> String {
    (file_size / 32).to_string()
}

/// 生成随机文件名
/// 
/// 返回格式为 "audio_{random_number}.wav" 的文件名
pub fn generate_random_file_name() -> String {
    let mut rng = rand::rng();
    let random_number: u32 = rng.random_range(1000..=9999);
    format!("audio_{random_number}.wav")
}

/// 从文件路径中提取任务名称
/// 
/// 如果提供了自定义名称，则使用自定义名称
/// 否则使用文件名（不含扩展名）作为任务名称
pub fn extract_task_name(audio_path: &Path, custom_name: Option<String>) -> String {
    match custom_name {
        Some(name) => name,
        None => audio_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .into(),
    }
}
