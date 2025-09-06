use async_trait::async_trait;
use mongodb::{Client, Collection, bson::doc};
use uuid::Uuid;
use crate::domain::{error::IngestionError, ports::DataRepository};

pub struct DocumentDBDataRepository {
    client: Client,
    database_name: String,
}

impl DocumentDBDataRepository {
    pub fn new(client: Client, database_name: String) -> Self {
        Self { client, database_name }
    }
}

#[async_trait]
impl DataRepository for DocumentDBDataRepository {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value], log_id: &str) -> Result<Vec<String>, IngestionError> {
        let db = self.client.database(&self.database_name);
        let collection: Collection<mongodb::bson::Document> = db.collection(target_table);
        
        let mut ids = Vec::new();
        let mut docs_to_insert = Vec::new();
        
        for doc in documents {
            let mut bson_doc = mongodb::bson::to_document(doc)
                .map_err(|e| IngestionError::Database(e.to_string()))?;
            
            // Add log_id to the document
            bson_doc.insert("log_id", log_id);
            
            docs_to_insert.push(bson_doc);
        }
        
        let result = collection.insert_many(docs_to_insert, None).await
            .map_err(|e| IngestionError::Database(e.to_string()))?;
        
        for id in result.inserted_ids.values() {
            ids.push(id.to_string());
        }
        
        Ok(ids)
    }
}