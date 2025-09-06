# Data Ingestion Application

A Rust application that runs on ECS, ingests files from S3 via SQS events, parses them into JSON documents, and stores them in configurable NoSQL databases using Hexagonal Architecture.

## Features

- **File Types Supported**: CSV, JSON, TXT, XML, XLS/XLSX
- **Databases**: MongoDB, CouchDB, DocumentDB
- **Architecture**: Hexagonal Architecture for clean separation of concerns
- **Configuration**: Database-driven configuration rules with regex pattern matching
- **Ingestion Logging**: Tracks processing start/end times, status, and error messages in `ingestion_logs` collection

## Prerequisites

### AWS Identity Provider Setup (for CI/CD)

1. **Deploy GitHub OIDC Identity Provider:**
```bash
aws cloudformation deploy \
  --template-file aws-oidc-setup.yaml \
  --stack-name github-oidc-setup \
  --parameter-overrides \
    GitHubOrg=your-github-username \
    GitHubRepo=data_ingestion \
  --capabilities CAPABILITY_NAMED_IAM
```

2. **Get the IAM Role ARN:**
```bash
aws cloudformation describe-stacks \
  --stack-name github-oidc-setup \
  --query 'Stacks[0].Outputs[?OutputKey==`RoleArn`].OutputValue' \
  --output text
```

3. **Configure GitHub Secrets:**
   - Go to your repository → Settings → Secrets and variables → Actions
   - Add the following secrets:
     - `AWS_ROLE_ARN`: The role ARN from step 2
     - `STAGING_MONGODB_URI`: MongoDB connection string for staging
     - `PRODUCTION_MONGODB_URI`: MongoDB connection string for production

## Setup

### Local Development

1. **Start external services:**
```bash
docker-compose up -d
```

2. **Run application locally:**
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

3. **Deploy LocalStack infrastructure:**
```bash
sam deploy --stack-name sam-app --parameter-overrides ImageUri=test-image --no-confirm-changeset
```

### Testing

**Test file processing:**
```bash
# Test all file types
./dev-test.sh

# Test specific file type
./dev-test.sh csv
./dev-test.sh json

# Test CSV without headers
./dev-test.sh csv --no-headers
```

**Verify processed data:**
```bash
# Check processed data
docker-compose exec mongodb mongosh ingestion_db --eval "db.csv_data.find().pretty()"

# Check ingestion logs
docker-compose exec mongodb mongosh ingestion_db --eval "db.ingestion_logs.find().pretty()"
```
### Production Deployment

**Environment Variables:**
- `DATABASE_TYPE`: Database type (mongodb, documentdb)
- `MONGODB_URI`: MongoDB connection string (if using MongoDB)
- `MONGODB_DATABASE`: Database name (if using MongoDB)
- `DOCUMENTDB_CONFIG_TABLE`: DocumentDB config table name (if using DocumentDB)
- `SQS_QUEUE_URL`: SQS queue URL for S3 events

**Manual deployment:**
```bash
aws cloudformation deploy \
  --template-file template.yaml \
  --stack-name data-ingestion \
  --parameter-overrides ImageUri=<ecr-image-uri> \
  --capabilities CAPABILITY_IAM
```

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

**Programmatic usage:**
```rust
let file = FileToProcess {
    bucket: "my-bucket".to_string(),
    key: "data/sample.csv".to_string(),
};

service.process_file(file).await?;
```

**File upload triggers:**
- Upload files to S3 bucket
- SQS events automatically trigger processing
- Results stored in configured database
- Processing logs tracked in `ingestion_logs` collection

## CI/CD Pipeline

The project uses GitHub Actions for automated testing, building, and deployment.

### Pipeline Features

- **Automated Testing**: Rust format check, Clippy linting, unit tests with MongoDB/LocalStack
- **Security Scanning**: Cargo audit, container vulnerability scanning with Trivy
- **Multi-Environment Deployment**: Separate staging and production environments
- **OIDC Authentication**: Secure AWS access without long-lived credentials
- **Dependency Management**: Automated updates via Dependabot

### Workflow Triggers

- **Pull Requests**: Run tests and security checks
- **Push to `develop`**: Deploy to staging environment
- **Push to `main`**: Deploy to production environment
- **Weekly Schedule**: Security vulnerability scans

### Environment Setup

1. **GitHub Environments**: Create `staging` and `production` environments in repository settings
2. **Branch Protection**: Configure branch protection rules for `main` branch
3. **Required Secrets**: Ensure all required secrets are configured (see Prerequisites)

### Manual Deployment

```bash
# Staging
aws cloudformation deploy \
  --template-file template.yaml \
  --stack-name data-ingestion-staging \
  --parameter-overrides ImageUri=<ecr-image-uri> \
  --capabilities CAPABILITY_IAM

# Production
aws cloudformation deploy \
  --template-file template.yaml \
  --stack-name data-ingestion-production \
  --parameter-overrides ImageUri=<ecr-image-uri> \
  --capabilities CAPABILITY_IAM
```

## Troubleshooting

**LocalStack Issues:**
```bash
# Restart LocalStack if deployment fails
docker-compose down && docker-compose up -d

# Check LocalStack logs
docker logs data_ingestion_localstack_1

# Check application logs (run locally)
./check-logs.sh
```

**Common Issues:**
- **IAM Role deletion errors**: Restart LocalStack to clear state
- **Bucket name conflicts**: Use unique bucket names in parameters
- **SQS queue not found**: Ensure LocalStack is fully started before deployment