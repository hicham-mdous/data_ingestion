use csv::ReaderBuilder;
use std::io::Cursor;
use tracing::{debug, info, error};
use crate::domain::error::IngestionError;

pub fn parse_csv(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    debug!("Creating CSV reader with headers enabled");
    let cursor = Cursor::new(bytes);
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(cursor);
    
    let headers = reader.headers()
        .map_err(|e| {
            error!("Failed to read CSV headers: {}", e);
            IngestionError::Parse(e.to_string())
        })?.clone();
    
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
            if let Some(header) = headers.get(i) {
                doc.insert(header.to_string(), serde_json::Value::String(field.to_string()));
            }
        }
        
        documents.push(serde_json::Value::Object(doc));
        
        if row_count % 1000 == 0 {
            debug!("Processed {} CSV rows", row_count);
        }
    }
    
    info!("Parsed {} rows from CSV", row_count);
    Ok(documents)
}