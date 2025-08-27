use tracing::{debug, error};
use crate::domain::error::IngestionError;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Value, Map};

pub fn parse_xml(bytes: &[u8]) -> Result<Vec<serde_json::Value>, IngestionError> {
    debug!("Parsing XML content");
    
    let mut reader = Reader::from_reader(bytes);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut records = Vec::new();
    let mut current_record: Option<Map<String, Value>> = None;
    let mut current_field = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "record" {
                    current_record = Some(Map::new());
                    // Extract attributes
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            if let Some(ref mut record) = current_record {
                                record.insert(key, Value::String(value));
                            }
                        }
                    }
                } else if current_record.is_some() {
                    current_field = name;
                }
            },
            Ok(Event::Text(e)) => {
                if let Some(ref mut record) = current_record {
                    if !current_field.is_empty() {
                        let text = e.unescape().unwrap_or_default().to_string();
                        record.insert(current_field.clone(), Value::String(text));
                    }
                }
            },
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "record" {
                    if let Some(record) = current_record.take() {
                        records.push(Value::Object(record));
                    }
                } else if current_record.is_some() {
                    current_field.clear();
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                error!("Error parsing XML: {}", e);
                return Err(IngestionError::Parse(e.to_string()));
            },
            _ => {},
        }
        buf.clear();
    }
    
    if records.is_empty() {
        error!("No records found in XML");
        return Err(IngestionError::Parse("No records found in XML".to_string()));
    }
    
    debug!("Parsed {} XML records", records.len());
    Ok(records)
}