use csv::ReaderBuilder;
use std::io::Cursor;
use tracing::{debug, info, error};
use crate::domain::error::IngestionError;

pub fn parse_csv(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    parse_csv_with_config(bytes, None)
}

pub fn parse_csv_with_config(bytes: &[u8], config: Option<&serde_json::Value>) -> Result<Vec<serde_json::Value>, IngestionError> {
    let cursor = Cursor::new(bytes);
    
    // Check if custom headers are provided in config
    let custom_headers = config
        .and_then(|c| c.get("headers"))
        .and_then(|h| h.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>());
    
    let has_headers = custom_headers.is_none();
    debug!("Creating CSV reader with headers={}, custom_headers={:?}", has_headers, custom_headers);
    
    let mut reader = ReaderBuilder::new().has_headers(has_headers).from_reader(cursor);
    
    let headers = if let Some(custom) = custom_headers {
        custom
    } else {
        reader.headers()
            .map_err(|e| {
                error!("Failed to read CSV headers: {}", e);
                IngestionError::Parse(e.to_string())
            })?
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    
    debug!("CSV headers: {:?}", headers);
    info!("Found {} columns in CSV", headers.len());
    
    let mut documents = Vec::new();
    let mut row_count = 0;
    
    for record in reader.records() {
        let record = record.map_err(|e| {
            error!("Failed to read CSV record at row {}: {}", row_count + 1, e);
            IngestionError::Parse(e.to_string())
        })?;
        
        row_count += 1;
        let mut doc = serde_json::Map::new();
        
        for (i, field) in record.iter().enumerate() {
            let fallback = format!("column_{}", i);
            let header = headers.get(i).map(|s| s.as_str()).unwrap_or(&fallback);
            doc.insert(header.to_string(), serde_json::Value::String(field.to_string()));
        }
        
        documents.push(serde_json::Value::Object(doc));
        
        if row_count % 1000 == 0 {
            debug!("Processed {} CSV rows", row_count);
        }
    }
    
    info!("Parsed {} rows from CSV", row_count);
    Ok(documents)
}