use async_trait::async_trait;
use mongodb::{Client, Collection, bson::Document};
use tracing::{debug, info, error};
use chrono::{DateTime, Utc};
use crate::domain::{error::IngestionError, models::{IngestionLog, IngestionStatus}, ports::LogRepository};

pub struct MongoLogRepository {
    client: Client,
    database: String,
}

impl MongoLogRepository {
    pub fn new(client: Client, database: String) -> Self {
        debug!("Initializing MongoDB log repository for database: {}", database);
        Self { client, database }
    }
}

#[async_trait]
impl LogRepository for MongoLogRepository {
    async fn insert_log(&self, log: &IngestionLog) -> Result<String, IngestionError> {
        debug!("Inserting ingestion log for file: {}", log.file_name);
        
        let collection: Collection<Document> = self.client.database(&self.database).collection("ingestion_logs");
        
        let doc = mongodb::bson::to_document(log)
            .map_err(|e| {
                error!("Failed to convert log to BSON: {}", e);
                IngestionError::Database(e.to_string())
            })?;
        
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| {
                error!("Failed to insert log for {}: {}", log.file_name, e);
                IngestionError::Database(e.to_string())
            })?;

        let log_id = if let mongodb::bson::Bson::ObjectId(oid) = result.inserted_id {
            oid.to_hex()
        } else {
            result.inserted_id.to_string()
        };
        info!("✅ Successfully logged ingestion for file: {} with ID: {}", log.file_name, log_id);
        Ok(log_id)
    }
    
    async fn update_log(&self, log_id: &str, end_time: DateTime<Utc>, status: IngestionStatus, message: Option<String>) -> Result<(), IngestionError> {
        use mongodb::bson::{doc, oid::ObjectId};
        
        debug!("Updating log with ID: {}", log_id);
        let collection: Collection<Document> = self.client.database(&self.database).collection("ingestion_logs");
        
        let object_id = ObjectId::parse_str(log_id)
            .map_err(|e| {
                error!("Failed to parse log_id '{}': {}", log_id, e);
                IngestionError::Database(format!("Invalid log_id: {}", e))
            })?;
        
        let update_doc = doc! {
            "$set": {
                "end_time": mongodb::bson::to_bson(&end_time).unwrap(),
                "status": mongodb::bson::to_bson(&status).unwrap(),
                "message": message
            }
        };
        
        debug!("Update document: {:?}", update_doc);
        
        let result = collection
            .update_one(doc! { "_id": object_id }, update_doc, None)
            .await
            .map_err(|e| {
                error!("Failed to update log {}: {}", log_id, e);
                IngestionError::Database(e.to_string())
            })?;
        
        debug!("Update result: matched={}, modified={}", result.matched_count, result.modified_count);
        
        if result.matched_count == 0 {
            error!("No log record found with ID: {}", log_id);
            return Err(IngestionError::Database(format!("Log record not found: {}", log_id)));
        }
        
        info!("✅ Successfully updated log with ID: {}", log_id);
        Ok(())
    }
}