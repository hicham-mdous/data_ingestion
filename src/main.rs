use data_ingestion::ecs_service::EcsService;
use tracing::{info, debug};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing with debug level
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("data_ingestion=debug".parse().unwrap())
            .add_directive("aws_sdk=warn".parse().unwrap())
            .add_directive("mongodb=info".parse().unwrap()))
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("Starting data ingestion application");
    debug!("Environment variables: DATABASE_TYPE={}, MONGODB_URI={}, SQS_QUEUE_URL={}", 
        std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "not set".to_string()),
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "not set".to_string()),
        std::env::var("SQS_QUEUE_URL").unwrap_or_else(|_| "not set".to_string())
    );
    
    let service = EcsService::new().await?;
    info!("ECS service initialized successfully");
    
    service.run().await
}
