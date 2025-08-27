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
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value]) -> Result<(), IngestionError> {
        let url = format!("{}/{}/_bulk_docs", self.base_url, target_table);
        
        let bulk_doc = serde_json::json!({
            "docs": documents
        });

        self.client
            .post(&url)
            .json(&bulk_doc)
            .send()
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        Ok(())
    }
}