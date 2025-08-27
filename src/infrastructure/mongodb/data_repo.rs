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
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value]) -> Result<(), IngestionError> {
        debug!("Inserting {} documents into collection: {}", documents.len(), target_table);
        
        if documents.is_empty() {
            info!("No documents to insert into {}", target_table);
            return Ok(());
        }
        
        let collection: Collection<Document> = self.client.database(&self.database).collection(target_table);
        debug!("Connected to collection: {}.{}", self.database, target_table);
        
        debug!("Converting {} JSON documents to BSON", documents.len());
        let docs: Vec<Document> = documents
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                mongodb::bson::to_document(doc)
                    .map_err(|e| {
                        error!("Failed to convert document {} to BSON: {}", i, e);
                        debug!("Problematic document: {}", serde_json::to_string_pretty(doc).unwrap_or_else(|_| "<invalid json>".to_string()));
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

        info!("✅ Successfully inserted {} documents into collection: {}", 
            result.inserted_ids.len(), target_table);
        debug!("Inserted document IDs: {:?}", result.inserted_ids);
        
        Ok(())
    }
}