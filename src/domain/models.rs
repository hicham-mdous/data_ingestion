use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfigRule {
    pub pattern: String,
    pub target_table: String,
    pub parser_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionLog {
    pub file_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: IngestionStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestionStatus {
    Success,
    Failed,
}