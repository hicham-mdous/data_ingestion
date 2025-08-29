use std::sync::Arc;
use aws_sdk_sqs::Client as SqsClient;
use serde_json::Value;
use tracing::{info, error, debug, warn};
use crate::{
    application::ingestion_service::IngestionService,
    domain::models::FileToProcess,
    infrastructure::{
        s3_adapter::S3Adapter,
        parser_adapter::ParserAdapter,
        mongodb::{config_repo::MongoConfigRepository, data_repo::MongoDataRepository, log_repo::MongoLogRepository},
        dynamodb::{config_repo::DynamoConfigRepository, data_repo::DynamoDataRepository},
    },
};

pub struct EcsService {
    service: IngestionService,
    sqs_client: SqsClient,
    queue_url: String,
}

impl EcsService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Initializing ECS service");
        
        debug!("Loading AWS configuration");
        let mut aws_config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest());
        
        // Configure endpoint for LocalStack if AWS_ENDPOINT_URL is set
        if let Ok(endpoint_url) = std::env::var("AWS_ENDPOINT_URL") {
            info!("Using custom AWS endpoint: {}", endpoint_url);
            aws_config_builder = aws_config_builder.endpoint_url(&endpoint_url);
        }
        
        let aws_config = aws_config_builder.load().await;
        debug!("AWS region: {:?}", aws_config.region());
        
        let mut s3_config = aws_sdk_s3::config::Builder::from(&aws_config);
        
        // Enable path-style addressing for LocalStack
        if std::env::var("AWS_ENDPOINT_URL").is_ok() {
            s3_config = s3_config.force_path_style(true);
        }
        
        let s3_client = aws_sdk_s3::Client::from_conf(s3_config.build());
        let sqs_client = SqsClient::new(&aws_config);
        debug!("AWS clients initialized");
        
        let queue_url = std::env::var("SQS_QUEUE_URL")
            .expect("SQS_QUEUE_URL environment variable is required");
        info!("Using SQS queue: {}", queue_url);
        
        let file_fetcher = Arc::new(S3Adapter::new(s3_client));
        let parser = Arc::new(ParserAdapter::new());
        debug!("S3 adapter and parser initialized");
        
        let db_type = std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "mongodb".to_string());
        info!("Using database type: {}", db_type);
        
        let service = match db_type.as_str() {
            "dynamodb" => {
                debug!("Initializing DynamoDB repositories");
                let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);
                let config_table = std::env::var("DYNAMODB_CONFIG_TABLE")
                    .unwrap_or_else(|_| "ingestion_config".to_string());
                info!("DynamoDB config table: {}", config_table);
                
                let config_repo = Arc::new(DynamoConfigRepository::new(dynamo_client.clone(), config_table));
                let data_repo = Arc::new(DynamoDataRepository::new(dynamo_client.clone()));
                // TODO: Implement DynamoDB log repository
                let log_repo = Arc::new(MongoLogRepository::new(mongodb::Client::with_uri_str("mongodb://localhost:27017").await?, "ingestion_db".to_string()));
                debug!("DynamoDB repositories initialized");
                
                IngestionService::new(file_fetcher, parser, config_repo, data_repo, log_repo)
            },
            _ => {
                debug!("Initializing MongoDB repositories");
                let mongo_uri = std::env::var("MONGODB_URI")
                    .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
                let mongo_db = std::env::var("MONGODB_DATABASE")
                    .unwrap_or_else(|_| "ingestion_db".to_string());
                info!("MongoDB URI: {}, Database: {}", mongo_uri, mongo_db);
                
                debug!("Connecting to MongoDB");
                let mongo_client = mongodb::Client::with_uri_str(&mongo_uri).await
                    .map_err(|e| {
                        error!("Failed to connect to MongoDB: {}", e);
                        e
                    })?;
                debug!("MongoDB client connected successfully");
                
                let config_repo = Arc::new(MongoConfigRepository::new(&mongo_client, &mongo_db));
                let data_repo = Arc::new(MongoDataRepository::new(mongo_client.clone(), mongo_db.clone()));
                let log_repo = Arc::new(MongoLogRepository::new(mongo_client, mongo_db));
                debug!("MongoDB repositories initialized");
                
                IngestionService::new(file_fetcher, parser, config_repo, data_repo, log_repo)
            }
        };
        
        debug!("ECS service initialization complete");
        Ok(Self { service, sqs_client, queue_url })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting ECS service, polling SQS queue: {}", self.queue_url);
        
        let mut poll_count = 0;
        loop {
            poll_count += 1;
            debug!("Polling SQS queue (attempt {})", poll_count);
            
            let response = self.sqs_client
                .receive_message()
                .queue_url(&self.queue_url)
                .max_number_of_messages(10)
                .wait_time_seconds(20)
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to receive messages from SQS: {}", e);
                    e
                })?;

            if let Some(messages) = response.messages {
                info!("Received {} messages from SQS", messages.len());
                
                for (i, message) in messages.iter().enumerate() {
                    debug!("Processing message {} of {}", i + 1, messages.len());
                    
                    if let Some(body) = &message.body {
                        debug!("Message body: {}", body);
                        
                        match self.process_message(body).await {
                            Ok(_) => {
                                info!("Successfully processed message {}", i + 1);
                            },
                            Err(e) => {
                                error!("Failed to process message {}: {}", i + 1, e);
                                debug!("Failed message body: {}", body);
                            }
                        }
                        
                        if let Some(receipt_handle) = &message.receipt_handle {
                            debug!("Deleting processed message from queue");
                            self.sqs_client
                                .delete_message()
                                .queue_url(&self.queue_url)
                                .receipt_handle(receipt_handle)
                                .send()
                                .await
                                .map_err(|e| {
                                    error!("Failed to delete message from SQS: {}", e);
                                    e
                                })?;
                            debug!("Message deleted from queue");
                        }
                    } else {
                        warn!("Received message without body");
                    }
                }
            } else {
                debug!("No messages received from SQS");
            }
        }
    }

    async fn process_message(&self, body: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Parsing S3 event message");
        let s3_event: Value = serde_json::from_str(body)
            .map_err(|e| {
                error!("Failed to parse S3 event JSON: {}", e);
                e
            })?;
        
        debug!("S3 event parsed successfully");
        
        if let Some(records) = s3_event["Records"].as_array() {
            info!("Processing {} S3 records", records.len());
            
            for (i, record) in records.iter().enumerate() {
                debug!("Processing S3 record {} of {}", i + 1, records.len());
                debug!("Record content: {}", serde_json::to_string_pretty(record).unwrap_or_else(|_| "<invalid json>".to_string()));
                
                if let (Some(bucket), Some(key)) = (
                    record["s3"]["bucket"]["name"].as_str(),
                    record["s3"]["object"]["key"].as_str(),
                ) {
                    info!("Processing file: s3://{}/{}", bucket, key);
                    
                    let file = FileToProcess {
                        bucket: bucket.to_string(),
                        key: key.to_string(),
                    };
                    
                    debug!("Calling ingestion service for file: {}/{}", bucket, key);
                    self.service.process_file(file).await
                        .map_err(|e| {
                            error!("Failed to process file {}/{}: {}", bucket, key, e);
                            e
                        })?;
                    
                    info!("Successfully processed file: {}/{}", bucket, key);
                } else {
                    warn!("S3 record missing bucket or key information");
                    debug!("Invalid record: {}", serde_json::to_string_pretty(record).unwrap_or_else(|_| "<invalid json>".to_string()));
                }
            }
        } else {
            warn!("S3 event contains no Records array");
            debug!("Event structure: {}", serde_json::to_string_pretty(&s3_event).unwrap_or_else(|_| "<invalid json>".to_string()));
        }
        
        debug!("Message processing completed");
        Ok(())
    }
}