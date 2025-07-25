use clap::Parser;
use iflypen_api_rs::{IflyrecClient, TranscriptionOptions, TranscriptionOrder, IflyrecError};
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashMap;
use std::io::Write;
use tokio::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;

#[derive(Clone, Parser)]
struct Args {
    /// 要上传的音频文件路径
    #[arg(
        short = 'f',
        long = "file",
        required_unless_present = "order_id",
        conflicts_with = "order_id",
        help = "音频文件路径"
    )]
    audio_file: Option<String>,

    /// 转录任务名称
    #[arg(short = 'n', long = "name", help = "为此次转录任务指定名称")]
    task_name: Option<String>,

    /// 热词设置（多个词用英文逗号分隔）
    #[arg(
        short = 'w',
        long = "hotwords",
        help = "指定热词，多个词用逗号分隔（如：Rust,WebRTC,AI）"
    )]
    hot_words: Option<String>,

    /// 语言设置
    #[arg(
        short = 'l',
        long = "lang",
        default_value = "cn",
        help = "指定音频语言类型（如：cn, en）"
    )]
    language: String,

    /// 是否需要短信通知
    #[arg(
        short = 's',
        long = "sms",
        help = "转录完成后是否通过短信通知",
        action = clap::ArgAction::SetTrue
    )]
    need_sms: bool,

    /// 数据库路径
    #[arg(
        short = 'd',
        long = "db",
        default_value = "Cookies",
        help = "SQLite 数据库路径，用于提取 session_id"
    )]
    database_path: String,

    /// 通过订单ID下载转写结果
    #[arg(
        short = 'o',
        long = "order-id",required_unless_present = "audio_file",
        conflicts_with_all = ["audio_file", "task_name", "hot_words", "language", "need_sms"],
        help = "指定已有订单ID直接下载结果"
    )]
    order_id: Option<String>,
}

/// 从数据库获取最频繁使用的session_id
fn get_most_frequent_session_id(database_path: &str) -> SqlResult<Option<String>> {
    let conn = Connection::open(database_path)?;
    let mut stmt = conn.prepare(
        "SELECT value FROM cookies WHERE name LIKE '%session%' AND host_key LIKE '%iflyrec%'",
    )?;

    let mut value_counts: HashMap<String, usize> = HashMap::new();
    let rows = stmt.query_map([], |row| {
        let value: String = row.get(0)?;
        Ok(value)
    })?;

    for value_result in rows {
        let value = value_result?;
        *value_counts.entry(value).or_insert(0) += 1;
    }

    let most_frequent = value_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(value, _)| value);

    Ok(most_frequent)
}

/// 处理热词：将英文逗号替换为中文逗号
fn process_hot_words(hot_words: Option<String>) -> String {
    hot_words
        .map(|words| words.replace(',', "，"))
        .unwrap_or_default()
}

/// 构建转录选项
fn build_transcription_options(args: &Args) -> Option<TranscriptionOptions> {
    let hot_words = process_hot_words(args.hot_words.clone());

    Some(TranscriptionOptions {
        need_sms: args.need_sms,
        hot_words,
        language: args.language.clone(),
    })
}

#[tokio::main]
async fn main() -> Result<(), IflyrecError> {
    let args = Args::parse();

    // 从数据库获取 session_id
    let session_id = match get_most_frequent_session_id(&args.database_path) {
        Ok(Some(id)) => {
            println!("✅ 成功获取 session_id");
            id
        }
        Ok(None) => {
            return Err(IflyrecError::AuthError("未找到有效的 session_id".to_string()));
        }
        Err(e) => {
            return Err(IflyrecError::DatabaseError(e));
        }
    };

    // 创建客户端
    let client = IflyrecClient::new(session_id);

    let order_id = if let Some(order_id) = args.order_id {
        order_id
    } else {
        println!("--- 开始转录任务 ---");
        let audio_file = args.audio_file.as_ref().unwrap();
        println!("音频文件: {audio_file}");

        if let Some(ref task_name) = args.task_name {
            println!("任务名称: {task_name}");
        }

        if args.need_sms {
            println!("短信通知: 已启用");
        }

        if let Some(ref hot_words) = args.hot_words {
            println!("热词设置成功: {hot_words}");
        }

        println!("语言设置: {}", args.language);
        println!("数据库路径: {}", args.database_path);

        // 构建转录选项
        let options = build_transcription_options(&args);

        // 提交转录任务
        let order_id = client
            .initiate_transcription_task(audio_file, args.task_name, options)
            .await?;
            
        println!("✅ 转录任务提交成功！");
        println!("订单 ID: {order_id}");
        order_id
    };
    println!("Waiting for the result...");
    println!("Program will exit after 1 minute.");
    let retry_strategy = ExponentialBackoff::from_millis(500)
        .max_delay(Duration::from_secs(10))
        .take(5); // 最多重试5次

    let order = Retry::spawn(retry_strategy, || async {
        match client.get_order(&order_id).await {
            Ok(order_option) => match order_option {
                Some(order) => {
                    println!("✅ 转录任务完成！");
                    Ok::<TranscriptionOrder, IflyrecError>(order)
                }
                None => Err(IflyrecError::OrderProcessing("⏳ 转录任务正在进行中...".to_string())),
            },
            Err(e) => Err(e),
        }
    })
    .await?;

    if order.order_status == "completed" {
        let result = client.get_order_result(&order).await?;

        let mut output_buffer = String::new();

        for paragraph in result.paragraphs {
            let words = paragraph.words;
            for word in words {
                output_buffer.push_str(&word.text);
            }
            output_buffer.push_str("\n\n");
        }

        // 写入文件
        let output_file_name = format!("{}.txt", order.order_name);
        let output_file_path = std::path::Path::new(&output_file_name);
        let mut output_file = std::fs::File::create(output_file_path)?;
        output_file.write_all(output_buffer.as_bytes())?;
        println!("✅ 转录结果已保存到文件: {output_file_name}");
    }

    Ok(())
}
