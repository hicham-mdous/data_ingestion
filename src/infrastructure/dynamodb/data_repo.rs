use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::{error::IngestionError, ports::DataRepository};

pub struct DynamoDataRepository {
    client: Client,
}

impl DynamoDataRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DataRepository for DynamoDataRepository {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value], log_id: &str) -> Result<Vec<String>, IngestionError> {
        let mut ids = Vec::new();
        
        for doc in documents {
            let mut item = HashMap::new();
            
            if let serde_json::Value::Object(obj) = doc {
                for (key, value) in obj {
                    let attr_value = match value {
                        serde_json::Value::String(s) => AttributeValue::S(s.clone()),
                        serde_json::Value::Number(n) => AttributeValue::N(n.to_string()),
                        serde_json::Value::Bool(b) => AttributeValue::Bool(*b),
                        _ => AttributeValue::S(value.to_string()),
                    };
                    item.insert(key.clone(), attr_value);
                }
            }
            
            // Add log_id to the item
            item.insert("log_id".to_string(), AttributeValue::S(log_id.to_string()));
            
            let response = self.client
                .put_item()
                .table_name(target_table)
                .set_item(Some(item))
                .send()
                .await
                .map_err(|e| IngestionError::Database(e.to_string()))?;
            
            // DynamoDB doesn't return IDs for put_item, so we generate a placeholder
            ids.push(format!("dynamo-{}", Uuid::new_v4()));
        }
        
        Ok(ids)
    }
}