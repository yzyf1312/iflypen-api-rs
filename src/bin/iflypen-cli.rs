use clap::Parser;
use iflypen_api_rs::api::{IflyrecClient, TranscriptionOptions, TranscriptionOrder};
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashMap;
use std::error::Error;
use tokio::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;

#[derive(Parser)]
struct Args {
    /// 要上传的音频文件路径
    #[arg(short = 'f', long = "file", help = "音频文件路径（如：data.mp3）")]
    audio_file: String,

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
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("--- 开始转录任务 ---");
    println!("音频文件: {}", args.audio_file);

    if let Some(ref task_name) = args.task_name {
        println!("任务名称: {}", task_name);
    }

    if args.need_sms {
        println!("短信通知: 已启用");
    }

    if let Some(ref hot_words) = args.hot_words {
        println!("热词设置成功: {hot_words}");
    }

    println!("语言设置: {}", args.language);
    println!("数据库路径: {}", args.database_path);

    // 从数据库获取 session_id
    let session_id = match get_most_frequent_session_id(&args.database_path) {
        Ok(Some(id)) => {
            println!("✅ 成功获取 session_id");
            id
        }
        Ok(None) => {
            eprintln!("❌ 错误: 未找到有效的 session_id");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ 数据库访问错误: {}", e);
            std::process::exit(1);
        }
    };

    // 创建客户端
    let client = IflyrecClient::new(session_id);

    // 构建转录选项
    let options = build_transcription_options(&args);

    // 提交转录任务
    let order_id = match client
        .initiate_transcription_task(&args.audio_file, args.task_name, options)
        .await
    {
        Ok(order_id) => {
            println!("✅ 转录任务提交成功！");
            println!("订单 ID: {}", order_id);
            order_id
        }
        Err(e) => {
            eprintln!("❌ 转录任务提交失败: {}", e);
            std::process::exit(1);
        }
    };
    println!("Waiting for the result...");
    println!("Program will exit after 1 minute.");
    let retry_strategy = ExponentialBackoff::from_millis(500)
        .max_delay(Duration::from_secs(10))
        .take(5); // 最多重试5次

    let order = Retry::spawn(retry_strategy, || async {
        match client.get_order(order_id.clone()).await {
            Ok(order_option) => match order_option {
                Some(order) => {
                    println!("✅ 转录任务完成！");
                    Ok::<TranscriptionOrder, Box<dyn Error>>(order)
                }
                None => Err("⏳ 转录任务正在进行中...".into()),
            },
            Err(e) => Err(format!("❌ 获取订单状态失败: {}", e).into()),
        }
    })
    .await?;

    println!("订单详细信息: {:#?}", order);

    Ok(())
}
