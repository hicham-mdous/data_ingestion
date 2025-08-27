use tracing::{debug, info, error};
use crate::domain::error::IngestionError;

pub fn parse_txt(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    debug!("Converting bytes to UTF-8 string");
    let content = String::from_utf8(bytes.to_vec())
        .map_err(|e| {
            error!("Failed to convert text file to UTF-8: {}", e);
            IngestionError::Parse(e.to_string())
        })?;
    
    let line_count = content.lines().count();
    debug!("Text file contains {} lines", line_count);
    
    let lines: Vec<serde_json::Value> = content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            serde_json::json!({
                "line_number": i + 1,
                "content": line
            })
        })
        .collect();
    
    info!("Converted {} text lines to JSON documents", line_count);
    Ok(lines)
}