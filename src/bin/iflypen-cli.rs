use iflypen_api_rs::api::IflyrecClient;
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashMap;
use std::error::Error;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let source_file = "data.mp3";
    println!("--- Starting ---");

    let session_id = get_most_frequent_session_id("Cookies").unwrap().unwrap();

    let client = IflyrecClient::new(session_id);

    match client
        .initiate_transcription_task(source_file, Some("Task from Rust".into()))
        .await
    {
        Ok(order_id) => {
            println!("✅ Finished! Order ID: {}", order_id);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    Ok(())
}
