use async_trait::async_trait;
use aws_sdk_s3::Client;
use tracing::{debug, info, error};
use crate::domain::{error::IngestionError, ports::FileFetcher};

pub struct S3Adapter {
    client: Client,
}

impl S3Adapter {
    pub fn new(client: Client) -> Self {
        debug!("Initializing S3 adapter");
        Self { client }
    }
}

#[async_trait]
impl FileFetcher for S3Adapter {
    async fn fetch_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, IngestionError> {
        debug!("Fetching file from S3: s3://{}/{}", bucket, key);
        
        let response = self.client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get object from S3 s3://{}/{}: {}", bucket, key, e);
                IngestionError::S3(e.to_string())
            })?;
        
        debug!("S3 GetObject response received for s3://{}/{}", bucket, key);
        debug!("Content type: {:?}", response.content_type());
        debug!("Content length: {:?}", response.content_length());
        debug!("Last modified: {:?}", response.last_modified());

        debug!("Reading response body for s3://{}/{}", bucket, key);
        let bytes = response.body
            .collect()
            .await
            .map_err(|e| {
                error!("Failed to read response body for s3://{}/{}: {}", bucket, key, e);
                IngestionError::S3(e.to_string())
            })?
            .into_bytes();

        info!("✅ Successfully fetched file s3://{}/{} - {} bytes", bucket, key, bytes.len());
        Ok(bytes.to_vec())
    }
}