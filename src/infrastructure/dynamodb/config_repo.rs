use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use regex::Regex;
use crate::domain::{
    error::IngestionError,
    models::IngestionConfigRule,
    ports::ConfigRepository,
};

pub struct DynamoConfigRepository {
    client: Client,
    table_name: String,
}

impl DynamoConfigRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }
}

#[async_trait]
impl ConfigRepository for DynamoConfigRepository {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError> {
        let response = self.client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        if let Some(items) = response.items {
            for item in items {
                if let (Some(pattern), Some(target_table)) = (
                    item.get("pattern").and_then(|v| v.as_s().ok()),
                    item.get("target_table").and_then(|v| v.as_s().ok()),
                ) {
                    let regex = Regex::new(pattern)
                        .map_err(|e| IngestionError::Config(e.to_string()))?;
                    
                    if regex.is_match(s3_key) {
                        let parser_config = item.get("parser_config")
                            .and_then(|v| v.as_s().ok())
                            .and_then(|s| serde_json::from_str(s).ok());
                        
                        return Ok(Some(IngestionConfigRule {
                            pattern: pattern.to_string(),
                            target_table: target_table.to_string(),
                            parser_config,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
}