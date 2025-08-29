use async_trait::async_trait;
use mongodb::{Client, Collection, bson::Document};
use tracing::{debug, info, error};
use crate::domain::{error::IngestionError, ports::DataRepository};

pub struct MongoDataRepository {
    client: Client,
    database: String,
}

impl MongoDataRepository {
    pub fn new(client: Client, database: String) -> Self {
        debug!("Initializing MongoDB data repository for database: {}", database);
        Self { client, database }
    }
}

#[async_trait]
impl DataRepository for MongoDataRepository {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value], log_id: &str) -> Result<Vec<String>, IngestionError> {
        debug!("Inserting {} documents into collection: {}", documents.len(), target_table);
        
        if documents.is_empty() {
            info!("No documents to insert into {}", target_table);
            return Ok(vec![]);
        }
        
        let collection: Collection<Document> = self.client.database(&self.database).collection(target_table);
        debug!("Connected to collection: {}.{}", self.database, target_table);
        
        debug!("Converting {} JSON documents to BSON and adding log_id: {}", documents.len(), log_id);
        let docs: Vec<Document> = documents
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let mut doc_with_log_id = doc.clone();
                if let serde_json::Value::Object(ref mut map) = doc_with_log_id {
                    map.insert("log_id".to_string(), serde_json::Value::String(log_id.to_string()));
                }
                mongodb::bson::to_document(&doc_with_log_id)
                    .map_err(|e| {
                        error!("Failed to convert document {} to BSON: {}", i, e);
                        debug!("Problematic document: {}", serde_json::to_string_pretty(&doc_with_log_id).unwrap_or_else(|_| "<invalid json>".to_string()));
                        e
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| IngestionError::Database(e.to_string()))?;
        
        debug!("Successfully converted {} documents to BSON", docs.len());
        
        debug!("Inserting documents into MongoDB collection: {}", target_table);
        let result = collection
            .insert_many(docs, None)
            .await
            .map_err(|e| {
                error!("Failed to insert documents into {}: {}", target_table, e);
                IngestionError::Database(e.to_string())
            })?;

        let ids: Vec<String> = result.inserted_ids.values()
            .map(|id| id.to_string())
            .collect();
        
        info!("✅ Successfully inserted {} documents into collection: {}", 
            ids.len(), target_table);
        debug!("Inserted document IDs: {:?}", ids);
        
        Ok(ids)
    }
}