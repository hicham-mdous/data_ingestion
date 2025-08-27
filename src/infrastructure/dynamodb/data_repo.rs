use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::collections::HashMap;
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
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value]) -> Result<(), IngestionError> {
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
            
            self.client
                .put_item()
                .table_name(target_table)
                .set_item(Some(item))
                .send()
                .await
                .map_err(|e| IngestionError::Database(e.to_string()))?;
        }
        
        Ok(())
    }
}