use std::sync::Arc;
use tracing::{info, debug, error, warn};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::domain::{
    error::IngestionError,
    models::{FileToProcess, IngestionConfigRule, IngestionLog, IngestionStatus},
    ports::{FileFetcher, DataParser, ConfigRepository, DataRepository, LogRepository},
};

pub struct IngestionService {
    file_fetcher: Arc<dyn FileFetcher>,
    data_parser: Arc<dyn DataParser>,
    config_repo: Arc<dyn ConfigRepository>,
    data_repo: Arc<dyn DataRepository>,
    log_repo: Arc<dyn LogRepository>,
}

impl IngestionService {
    pub fn new(
        file_fetcher: Arc<dyn FileFetcher>,
        data_parser: Arc<dyn DataParser>,
        config_repo: Arc<dyn ConfigRepository>,
        data_repo: Arc<dyn DataRepository>,
        log_repo: Arc<dyn LogRepository>,
    ) -> Self {
        Self {
            file_fetcher,
            data_parser,
            config_repo,
            data_repo,
            log_repo,
        }
    }

    pub async fn process_file(&self, file: FileToProcess) -> Result<(), IngestionError> {
        let start_time = Utc::now();
        let file_name = format!("{}/{}", file.bucket, file.key);
        
        info!("Starting file processing: s3://{}", file_name);
        
        self.process_file_internal(&file, start_time).await
    }
    
    async fn process_file_internal(&self, file: &FileToProcess, start_time: DateTime<Utc>) -> Result<(), IngestionError> {
        debug!("File details - bucket: {}, key: {}", file.bucket, file.key);

        // Step 1: Find matching configuration
        debug!("Step 1: Finding matching configuration for key: {}", file.key);
        let config = self.find_matching_config(&file.key).await
            .map_err(|e| {
                error!("Failed to find matching config for {}: {}", file.key, e);
                e
            })?;
        info!("Found matching config - target table: {}, pattern: {}", config.target_table, config.pattern);
        
        // Step 2: Fetch file from S3
        debug!("Step 2: Fetching file from S3: {}/{}", file.bucket, file.key);
        let file_bytes = self.file_fetcher.fetch_file(&file.bucket, &file.key).await
            .map_err(|e| {
                error!("Failed to fetch file {}/{}: {}", file.bucket, file.key, e);
                e
            })?;
        info!("Successfully fetched file, size: {} bytes", file_bytes.len());
        
        // Step 3: Extract file type
        let file_type = self.extract_file_type(&file.key);
        debug!("Step 3: Detected file type: {}", file_type);
        
        // Step 4: Parse file content
        debug!("Step 4: Parsing file content with type: {} and config: {:?}", file_type, config.parser_config);
        let documents = self.data_parser.parse_with_config(&file_bytes, &file_type, config.parser_config.as_ref()).await
            .map_err(|e| {
                error!("Failed to parse file {}: {}", file.key, e);
                e
            })?;
        info!("Successfully parsed {} documents from file", documents.len());
        
        // Step 5: Add file_name to each document and store
        debug!("Step 5: Adding file_name and storing {} documents to table: {}", documents.len(), config.target_table);
        let file_name = format!("{}/{}", file.bucket, file.key);
        let mut documents_with_filename: Vec<serde_json::Value> = documents
            .into_iter()
            .map(|mut doc| {
                if let serde_json::Value::Object(ref mut map) = doc {
                    map.insert("file_name".to_string(), serde_json::Value::String(file_name.clone()));
                }
                doc
            })
            .collect();
        
        // Create initial log entry to get log_id
        let log = IngestionLog {
            file_name: format!("{}/{}", file.bucket, file.key),
            start_time,
            end_time: None,
            status: IngestionStatus::Success,
            message: None,
        };
        let log_id = self.log_repo.insert_log(&log).await
            .map_err(|e| {
                error!("Failed to create log entry for {}: {}", file.key, e);
                e
            })?;
        
        let processing_result: Result<(), IngestionError> = async {
            let _inserted_ids = self.data_repo.insert_documents(&config.target_table, &documents_with_filename, &log_id).await
                .map_err(|e| {
                    error!("Failed to store documents for {}: {}", file.key, e);
                    e
                })?;
            
            info!("✅ Successfully processed file {}/{} - {} documents stored in {}", 
                file.bucket, file.key, documents_with_filename.len(), config.target_table);
            Ok::<(), IngestionError>(())
        }.await;
        
        // Update log with final status
        let (status, message) = match &processing_result {
            Ok(_) => (IngestionStatus::Success, Some("File processed successfully".to_string())),
            Err(e) => (IngestionStatus::Failed, Some(e.to_string())),
        };
        
        let _ = self.log_repo.update_log(&log_id, Utc::now(), status, message).await;
        
        processing_result
    }

    async fn find_matching_config(&self, s3_key: &str) -> Result<IngestionConfigRule, IngestionError> {
        debug!("Searching for configuration rule matching key: {}", s3_key);
        
        match self.config_repo.get_config_for_key(s3_key).await {
            Ok(Some(config)) => {
                debug!("Found matching config rule: pattern='{}', target_table='{}'", 
                    config.pattern, config.target_table);
                Ok(config)
            },
            Ok(None) => {
                warn!("No configuration rule found for key: {}", s3_key);
                Err(IngestionError::NoMatchingRule(s3_key.to_string()))
            },
            Err(e) => {
                error!("Error retrieving configuration for key {}: {}", s3_key, e);
                Err(e)
            }
        }
    }

    fn extract_file_type(&self, key: &str) -> String {
        let file_type = key.split('.').last().unwrap_or("").to_lowercase();
        debug!("Extracted file type '{}' from key: {}", file_type, key);
        
        if file_type.is_empty() {
            warn!("No file extension found in key: {}", key);
        }
        
        file_type
    }
}