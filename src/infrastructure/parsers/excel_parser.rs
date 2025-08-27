use calamine::{Reader, Xlsx};
use std::io::Cursor;
use tracing::{debug, info, error};
use crate::domain::error::IngestionError;

pub fn parse_excel(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    debug!("Parsing Excel file");
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)
        .map_err(|e| {
            error!("Failed to open Excel file: {}", e);
            IngestionError::Parse(e.to_string())
        })?;
    
    let mut documents = Vec::new();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        debug!("Processing Excel worksheet");
        let mut rows = range.rows();
        let headers: Vec<String> = if let Some(header_row) = rows.next() {
            header_row.iter().map(|cell| cell.to_string()).collect()
        } else {
            debug!("No header row found in Excel file");
            return Ok(documents);
        };

        debug!("Excel headers: {:?}", headers);
        let mut row_count = 0;

        for row in rows {
            let mut doc = serde_json::Map::new();
            for (i, cell) in row.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    doc.insert(header.clone(), serde_json::Value::String(cell.to_string()));
                }
            }
            documents.push(serde_json::Value::Object(doc));
            row_count += 1;
        }
        
        info!("Parsed {} rows from Excel file", row_count);
    } else {
        debug!("No worksheet found in Excel file");
    }
    
    Ok(documents)
}