use crate::error::Result;
use rand::{Rng, distr::uniform::SampleUniform};
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

pub fn random_in_range<T>(range: [T; 2]) -> T
where
    T: SampleUniform + PartialOrd + Copy,
{
    let start = range[0];
    let end = range[1];

    let inclusive_range = if start <= end {
        start..=end
    } else {
        end..=start
    };

    rand::rng().random_range(inclusive_range)
}
