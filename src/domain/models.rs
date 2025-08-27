use serde::{Deserialize, Serialize};

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