use std::sync::Arc;
use tracing::{info, debug, error, warn};
use crate::domain::{
    error::IngestionError,
    models::{FileToProcess, IngestionConfigRule},
    ports::{FileFetcher, DataParser, ConfigRepository, DataRepository},
};

pub struct IngestionService {
    file_fetcher: Arc<dyn FileFetcher>,
    data_parser: Arc<dyn DataParser>,
    config_repo: Arc<dyn ConfigRepository>,
    data_repo: Arc<dyn DataRepository>,
}

impl IngestionService {
    pub fn new(
        file_fetcher: Arc<dyn FileFetcher>,
        data_parser: Arc<dyn DataParser>,
        config_repo: Arc<dyn ConfigRepository>,
        data_repo: Arc<dyn DataRepository>,
    ) -> Self {
        Self {
            file_fetcher,
            data_parser,
            config_repo,
            data_repo,
        }
    }

    pub async fn process_file(&self, file: FileToProcess) -> Result<(), IngestionError> {
        info!("Starting file processing: s3://{}/{}", file.bucket, file.key);
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
        debug!("Step 4: Parsing file content with type: {}", file_type);
        let documents = self.data_parser.parse(&file_bytes, &file_type).await
            .map_err(|e| {
                error!("Failed to parse file {}: {}", file.key, e);
                e
            })?;
        info!("Successfully parsed {} documents from file", documents.len());
        
        // Step 5: Store documents
        debug!("Step 5: Storing {} documents to table: {}", documents.len(), config.target_table);
        self.data_repo.insert_documents(&config.target_table, &documents).await
            .map_err(|e| {
                error!("Failed to store documents for {}: {}", file.key, e);
                e
            })?;
        
        info!("✅ Successfully processed file {}/{} - {} documents stored in {}", 
            file.bucket, file.key, documents.len(), config.target_table);
        Ok(())
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