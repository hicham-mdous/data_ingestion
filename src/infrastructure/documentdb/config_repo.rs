use async_trait::async_trait;
use mongodb::{Client, Collection, bson::doc};
use futures_util::TryStreamExt;
use regex::Regex;
use crate::domain::{
    error::IngestionError,
    models::IngestionConfigRule,
    ports::ConfigRepository,
};

pub struct DocumentDBConfigRepository {
    client: Client,
    database_name: String,
    collection_name: String,
}

impl DocumentDBConfigRepository {
    pub fn new(client: Client, database_name: String, collection_name: String) -> Self {
        Self { client, database_name, collection_name }
    }
}

#[async_trait]
impl ConfigRepository for DocumentDBConfigRepository {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError> {
        let db = self.client.database(&self.database_name);
        let collection: Collection<mongodb::bson::Document> = db.collection(&self.collection_name);
        
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e| IngestionError::Database(e.to_string()))?;
        
        while let Some(item) = cursor.try_next().await
            .map_err(|e| IngestionError::Database(e.to_string()))? {
            if let (Some(pattern), Some(target_table)) = (
                item.get_str("pattern").ok(),
                item.get_str("target_table").ok(),
            ) {
                let regex = Regex::new(pattern)
                    .map_err(|e| IngestionError::Config(e.to_string()))?;
                
                if regex.is_match(s3_key) {
                    let parser_config = item.get_str("parser_config").ok()
                        .and_then(|s| serde_json::from_str(s).ok());
                    
                    return Ok(Some(IngestionConfigRule {
                        pattern: pattern.to_string(),
                        target_table: target_table.to_string(),
                        parser_config,
                    }));
                }
            }
        }
        
        Ok(None)
    }
}