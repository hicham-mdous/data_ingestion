Instructions for AI Agent: Build Rust Data Ingestion Application
Project Overview
Create a Rust application that ingests files from S3, parses them into JSON documents, and stores them in a configurable NoSQL database. The application must use Hexagonal Architecture and retrieve configuration mappings from a database table.

Core Requirements
Read files from S3 bucket using AWS SDK

Parse multiple file types: CSV, TXT, XML, JSON, XLS, PDF

Store parsed JSON documents in NoSQL database

Use database table for configuration mappings (not file-based)

Implement Hexagonal Architecture for clear separation of concerns

Support multiple database technologies (MongoDB, CouchDB)

Architecture Specifications
Hexagonal Architecture Structure
text
src/
├── domain/                 # The core business logic (independent of infrastructure)
│   ├── models.rs          # Data structures
│   ├── error.rs           # Error types
│   └── ports.rs           # Traits defining interfaces
├── application/           # Orchestration logic
│   └── ingestion_service.rs
└── infrastructure/        # External implementations
    ├── s3_adapter.rs
    ├── mongodb/
    │   ├── config_repo.rs
    │   └── data_repo.rs
    └── couchdb/
        ├── config_repo.rs
        └── data_repo.rs
Key Components to Implement
1. Domain Layer (ports.rs)
rust
// File fetching port
#[async_trait]
pub trait FileFetcher: Send + Sync {
    async fn fetch_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, IngestionError>;
}

// Data parsing port
#[async_trait]
pub trait DataParser: Send + Sync {
    async fn parse(&self, file_bytes: &[u8], file_type: &str) -> Result<Vec<serde_json::Value>, IngestionError>;
}

// Configuration repository port
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_config_for_key(&self, s3_key: &str) -> Result<Option<IngestionConfigRule>, IngestionError>;
}

// Data repository port
#[async_trait]
pub trait DataRepository: Send + Sync {
    async fn insert_documents(&self, target_table: &str, documents: &[serde_json::Value]) -> Result<(), IngestionError>;
}
2. Domain Models (models.rs)
rust
pub struct IngestionConfigRule {
    pub pattern: String,        // Regex pattern to match S3 keys
    pub target_table: String,   // Target collection/table
    pub parser_config: Option<serde_json::Value>, // Optional parser configuration
}

pub struct FileToProcess {
    pub bucket: String,
    pub key: String,
}
3. Application Service (ingestion_service.rs)
Create a service that:

Takes a FileToProcess object

Queries the ConfigRepository for a matching rule

Fetches the file from S3 using FileFetcher

Parses the file using DataParser

Stores results using DataRepository

4. Infrastructure Adapters
S3 Adapter: Implement FileFetcher using AWS SDK

Parser Adapter: Implement DataParser with support for all required file types

MongoDB Adapters: Implement both ConfigRepository and DataRepository

CouchDB Adapters: Implement both ConfigRepository and DataRepository

Implementation Details
Database Schema for Config Mappings
Create a collection/table named ingestion_config with documents/records containing:

pattern (string): Regex pattern to match S3 keys

target_table (string): Destination table/collection

parser_config (object, optional): Parser-specific settings

File Processing Logic
For each file to process:

Query the ingestion_config table to find a rule where the regex pattern matches the S3 key

If no rule is found, return an appropriate error

Fetch file content from S3

Determine file type from extension

Parse according to file type

Insert parsed documents into the target table specified in the rule

Error Handling
Create a comprehensive error enum covering:

Configuration errors (no matching rule)

S3 errors (fetch failures)

Parsing errors (file format issues)

Database errors (connection, insertion failures)

Dependencies
Include these crates in Cargo.toml:

aws-sdk-s3 for S3 access

csv for CSV parsing

serde_json for JSON handling

quick-xml for XML parsing

calamine for Excel parsing

pdf-extract for PDF extraction

mongodb for MongoDB support

reqwest for CouchDB support

async-trait for async trait methods

tokio as async runtime

thiserror for error handling

tracing for logging

regex for pattern matching

Configuration
Use a config.yaml file for:

AWS region and credentials

Database connection strings

Active database selection (MongoDB or CouchDB)

Additional Requirements
Include a database migration script to create the ingestion_config table

Provide example configuration records for different file types

Implement comprehensive logging using the tracing crate

Make all database operations asynchronous

Ensure the core domain has no dependencies on external crates

Expected Output
The AI should generate:

Complete Rust application with all specified modules

Cargo.toml with all dependencies

Configuration files

Database migration script

Example usage code

Documentation comments

Validation Criteria
The core domain module must not depend on any infrastructure crates

Adding a new database technology should only require new adapter implementations

The application should handle all specified file types

Configuration should be retrieved from database, not files

Comprehensive error handling should be implemented

Please generate the complete application according to these specifications.