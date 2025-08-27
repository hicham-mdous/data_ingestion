# Data Ingestion Application

A Rust application that runs on ECS, ingests files from S3 via SQS events, parses them into JSON documents, and stores them in configurable NoSQL databases using Hexagonal Architecture.

## Features

- **File Types Supported**: CSV, JSON, TXT, XML, XLS/XLSX
- **Databases**: MongoDB, CouchDB, DynamoDB
- **Architecture**: Hexagonal Architecture for clean separation of concerns
- **Configuration**: Database-driven configuration rules with regex pattern matching

## Setup

### Local Development
```

### Development Testing (Local App)
1. Start external services:
```bash
docker-compose up -d
```

2. Run application locally:
```bash
export DATABASE_TYPE=mongodb
export MONGODB_URI=mongodb://localhost:27017
export MONGODB_DATABASE=ingestion_db
export SQS_QUEUE_URL=http://localhost:4566/000000000000/test-queue
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_DEFAULT_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:4566
export RUST_LOG=debug
export RUST_BACKTRACE=1
cargo watch -c -w src -x run
```


3. Test file processing:
```bash
./dev-test.sh
```

4. Verify data:
```bash
docker-compose  exec mongodb mongosh ingestion_db --eval "db.csv_data.find().pretty()"
```

5. Test different file types:
```bash
# JSON file
echo '[{"name":"test","value":123}]' > test.json
aws --endpoint-url=http://localhost:4566 s3 cp test.json s3://test-bucket/data/test.json

# Text file
echo "Line 1\nLine 2\nLine 3" > test.txt
aws --endpoint-url=http://localhost:4566 s3 cp test.txt s3://test-bucket/logs/test.txt

### ECS Deployment
1. Build and push to ECR:
```bash
./build.sh
```

2. Deploy using CloudFormation:
```bash
aws cloudformation deploy --template-file ecs-template.yaml --stack-name data-ingestion --parameter-overrides ImageUri=<your-ecr-uri> --capabilities CAPABILITY_IAM
```

3. Environment variables:
- `DATABASE_TYPE`: Database type (mongodb, dynamodb)
- `MONGODB_URI`: MongoDB connection string (if using MongoDB)
- `MONGODB_DATABASE`: Database name (if using MongoDB)
- `DYNAMODB_CONFIG_TABLE`: DynamoDB config table name (if using DynamoDB)
- `SQS_QUEUE_URL`: SQS queue URL for S3 events

## Architecture

```
src/
├── domain/                 # Core business logic
│   ├── models.rs          # Data structures
│   ├── error.rs           # Error types
│   └── ports.rs           # Interface traits
├── application/           # Orchestration logic
│   └── ingestion_service.rs
└── infrastructure/        # External implementations
    ├── s3_adapter.rs
    ├── parser_adapter.rs
    ├── mongodb/
    └── couchdb/
```

## Configuration Rules

The application uses database-stored configuration rules that match S3 keys using regex patterns:

- `pattern`: Regex to match S3 keys
- `target_table`: Destination collection/table
- `parser_config`: Optional parser settings

## Usage

```rust
let file = FileToProcess {
    bucket: "my-bucket".to_string(),
    key: "data/sample.csv".to_string(),
};

service.process_file(file).await?;
```