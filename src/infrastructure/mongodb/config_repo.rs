use async_trait::async_trait;
use mongodb::{Client, Collection, bson::doc};
use regex::Regex;
use tracing::{debug, info, warn, error};
use crate::domain::{
    error::IngestionError,
    models::IngestionConfigRule,
    ports::ConfigRepository,
};

pub struct MongoConfigRepository {
    collection: Collection<IngestionConfigRule>,
}

impl MongoConfigRepository {
    pub fn new(client: &Client, database: &str) -> Self {
        debug!("Initializing MongoDB config repository for database: {}", database);
        let collection = client.database(database).collection("ingestion_config");
        debug!("MongoDB config repository initialized");
        Self { collection }
    }
}

#[async_trait]
impl ConfigRepository for MongoConfigRepository {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError> {
        debug!("Searching for config rule matching S3 key: {}", s3_key);
        
        let mut cursor = self.collection
            .find(doc! {}, None)
            .await
            .map_err(|e| {
                error!("Failed to query config collection: {}", e);
                IngestionError::Database(e.to_string())
            })?;

        let mut matching_rules = Vec::new();
        let mut rules_checked = 0;
        
        while cursor.advance().await.map_err(|e| {
            error!("Failed to advance cursor: {}", e);
            IngestionError::Database(e.to_string())
        })? {
            let rule = cursor.deserialize_current()
                .map_err(|e| {
                    error!("Failed to deserialize config rule: {}", e);
                    IngestionError::Database(e.to_string())
                })?;
            
            rules_checked += 1;
            debug!("Checking rule {}: pattern='{}', target_table='{}'", 
                rules_checked, rule.pattern, rule.target_table);
            
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| {
                    error!("Invalid regex pattern '{}': {}", rule.pattern, e);
                    IngestionError::Config(e.to_string())
                })?;
            
            if regex.is_match(s3_key) {
                debug!("✅ Rule pattern '{}' matches key '{}'", rule.pattern, s3_key);
                matching_rules.push(rule);
            } else {
                debug!("❌ Rule pattern '{}' does not match key '{}'", rule.pattern, s3_key);
            }
        }
        
        if matching_rules.is_empty() {
            warn!("No matching configuration rule found for '{}' after checking {} rules", s3_key, rules_checked);
            return Ok(None);
        }
        
        // Select the most specific rule (longest pattern)
        let best_rule = matching_rules
            .into_iter()
            .max_by_key(|rule| rule.pattern.len())
            .unwrap();
        
        info!("✅ Best matching rule for '{}': pattern='{}', target_table='{}'", 
            s3_key, best_rule.pattern, best_rule.target_table);
        
        Ok(Some(best_rule))
    }
}