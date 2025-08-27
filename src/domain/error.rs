use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("S3 error: {0}")]
    S3(String),
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("No matching configuration rule found for key: {0}")]
    NoMatchingRule(String),
}