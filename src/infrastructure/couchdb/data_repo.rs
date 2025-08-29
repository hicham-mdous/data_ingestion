use async_trait::async_trait;
use reqwest::Client;
use crate::domain::{error::IngestionError, ports::DataRepository};

pub struct CouchDataRepository {
    client: Client,
    base_url: String,
    database: String,
}

impl CouchDataRepository {
    pub fn new(base_url: String, database: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            database,
        }
    }
}

#[async_trait]
impl DataRepository for CouchDataRepository {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value], log_id: &str) -> Result<Vec<String>, IngestionError> {
        let url = format!("{}/{}/_bulk_docs", self.base_url, target_table);
        
        let docs_with_log_id: Vec<serde_json::Value> = documents
            .iter()
            .map(|doc| {
                let mut doc_with_log_id = doc.clone();
                if let serde_json::Value::Object(ref mut map) = doc_with_log_id {
                    map.insert("log_id".to_string(), serde_json::Value::String(log_id.to_string()));
                }
                doc_with_log_id
            })
            .collect();
        
        let bulk_doc = serde_json::json!({
            "docs": docs_with_log_id
        });

        let response = self.client
            .post(&url)
            .json(&bulk_doc)
            .send()
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        let ids = result.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|item| item.get("id").and_then(|id| id.as_str()))
            .map(|id| id.to_string())
            .collect();

        Ok(ids)
    }
}