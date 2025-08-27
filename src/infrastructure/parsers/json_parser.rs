use tracing::{debug, error};
use crate::domain::error::IngestionError;

pub fn parse_json(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    debug!("Parsing JSON content");
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| {
            error!("Failed to parse JSON: {}", e);
            debug!("JSON content preview: {}", 
                String::from_utf8_lossy(&bytes[..std::cmp::min(200, bytes.len())])
            );
            IngestionError::Parse(e.to_string())
        })?;
    
    let result = match value {
        serde_json::Value::Array(arr) => {
            debug!("JSON contains array with {} elements", arr.len());
            Ok(arr)
        },
        single => {
            debug!("JSON contains single object, wrapping in array");
            Ok(vec![single])
        }
    };
    
    result
}