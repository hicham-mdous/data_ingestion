use async_trait::async_trait;
use crate::domain::{error::IngestionError, models::IngestionConfigRule};

#[async_trait]
pub trait FileFetcher: Send + Sync {
    async fn fetch_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, IngestionError>;
}

#[async_trait]
pub trait DataParser: Send + Sync {
    async fn parse(&self, file_bytes: &[u8], file_type: &str) -> Result<Vec<serde_json::Value>, IngestionError>;
}

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError>;
}

#[async_trait]
pub trait DataRepository: Send + Sync {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value]) -> Result<(), IngestionError>;
}