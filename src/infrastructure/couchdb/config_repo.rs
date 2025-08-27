use async_trait::async_trait;
use reqwest::Client;
use regex::Regex;
use serde_json::Value;
use crate::domain::{
    error::IngestionError,
    models::IngestionConfigRule,
    ports::ConfigRepository,
};

pub struct CouchConfigRepository {
    client: Client,
    base_url: String,
    database: String,
}

impl CouchConfigRepository {
    pub fn new(base_url: String, database: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            database,
        }
    }
}

#[async_trait]
impl ConfigRepository for CouchConfigRepository {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError> {
        let url = format!("{}/{}/_all_docs?include_docs=true", self.base_url, self.database);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        let result: Value = response
            .json()
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        if let Some(rows) = result["rows"].as_array() {
            for row in rows {
                if let Some(doc) = row["doc"].as_object() {
                    let rule: IngestionConfigRule = serde_json::from_value(Value::Object(doc.clone()))
                        .map_err(|e| IngestionError::Database(e.to_string()))?;
                    
                    let regex = Regex::new(&rule.pattern)
                        .map_err(|e| IngestionError::Config(e.to_string()))?;
                    
                    if regex.is_match(s3_key) {
                        return Ok(Some(rule));
                    }
                }
            }
        }
        
        Ok(None)
    }
}