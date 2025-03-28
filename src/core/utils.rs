use crate::error::Result;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncBufReadExt;

pub async fn read_lines(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = tokio::fs::File::open(path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut contents = vec![];
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            contents.push(trimmed.to_string());
        }
    }

    Ok(contents)
}

pub fn get_timestamp_utc_now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
