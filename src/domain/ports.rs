use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::{error::IngestionError, models::{IngestionConfigRule, IngestionLog, IngestionStatus}};

#[async_trait]
pub trait FileFetcher: Send + Sync {
    async fn fetch_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, IngestionError>;
}

#[async_trait]
pub trait DataParser: Send + Sync {
    async fn parse(&self, file_bytes: &[u8], file_type: &str) -> Result<Vec<serde_json::Value>, IngestionError>;
    async fn parse_with_config(&self, file_bytes: &[u8], file_type: &str, config: Option<&serde_json::Value>) -> Result<Vec<serde_json::Value>, IngestionError>;
}

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError>;
}

#[async_trait]
pub trait DataRepository: Send + Sync {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value], log_id: &str) -> Result<Vec<String>, IngestionError>;
}

#[async_trait]
pub trait LogRepository: Send + Sync {
    async fn insert_log(&self, log: &IngestionLog) -> Result<String, IngestionError>;
    async fn update_log(&self, log_id: &str, end_time: DateTime<Utc>, status: IngestionStatus, message: Option<String>) -> Result<(), IngestionError>;
}