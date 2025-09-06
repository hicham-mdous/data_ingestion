#!/bin/bash

# Check for file type argument and options
FILE_TYPE=${1:-"all"}
NO_HEADERS=false

# Check for --no-headers option
if [[ "$2" == "--no-headers" ]] || [[ "$1" == "--no-headers" && "$2" != "" ]]; then
    NO_HEADERS=true
    if [[ "$1" == "--no-headers" ]]; then
        FILE_TYPE=${2:-"csv"}
    fi
fi

if [[ "$FILE_TYPE" != "csv" && "$FILE_TYPE" != "txt" && "$FILE_TYPE" != "json" && "$FILE_TYPE" != "xml" && "$FILE_TYPE" != "xls" && "$FILE_TYPE" != "pdf" && "$FILE_TYPE" != "all" ]]; then
    echo "Usage: $0 [csv|txt|json|xml|xls|pdf|all] [--no-headers]"
    echo "       $0 --no-headers [csv]"
    exit 1
fi

echo "🧪 Running development tests for: $FILE_TYPE"

# Set environment variables for local development
export DATABASE_TYPE=mongodb
export MONGODB_URI=mongodb://localhost:27017
export MONGODB_DATABASE=ingestion_db
export SQS_QUEUE_URL=http://localhost:4566/000000000000/test-queue
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_DEFAULT_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:4566

# Bucket configuration
BUCKET_NAME=${BUCKET_NAME:-"data-ingestion-bucket"}

# Check if bucket exists, create if not
echo "🪣 Checking S3 bucket: $BUCKET_NAME"
if ! aws --endpoint-url=http://localhost:4566 s3 ls s3://$BUCKET_NAME >/dev/null 2>&1; then
    echo "📦 Creating S3 bucket: $BUCKET_NAME"
    aws --endpoint-url=http://localhost:4566 s3 mb s3://$BUCKET_NAME
else
    echo "✅ S3 bucket exists: $BUCKET_NAME"
fi

test_csv() {
    mkdir -p data
    
    if [[ "$NO_HEADERS" == "true" ]]; then
        echo "📄 Creating CSV test file (no headers)..."
        echo "John,30,NYC
Jane,25,LA
Bob,35,Chicago" > data/test_no_headers.csv
        aws --endpoint-url=http://localhost:4566 s3 cp data/test_no_headers.csv s3://$BUCKET_NAME/data/test_no_headers.csv
        echo "✅ CSV file (no headers) uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.csv_data.find().pretty()\""
    else
        echo "📄 Creating CSV test file (with headers)..."
        echo "name,age,city
John,30,NYC
Jane,25,LA
Bob,35,Chicago" > data/test.csv
        aws --endpoint-url=http://localhost:4566 s3 cp data/test.csv s3://$BUCKET_NAME/data/test.csv
        echo "✅ CSV file (with headers) uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.csv_data.find().pretty()\""
    fi
}

test_json() {
    echo "📄 Creating JSON test file..."
    mkdir -p data
    echo '[{"name":"Alice","value":100},{"name":"Bob","value":200}]' > data/test.json
    aws --endpoint-url=http://localhost:4566 s3 cp data/test.json s3://$BUCKET_NAME/data/test.json
    echo "✅ JSON file uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.json_data.find().pretty()\""
}

test_txt() {
    echo "📄 Creating TXT test file..."
    mkdir -p data
    echo "Log entry 1: Application started
Log entry 2: Processing data
Log entry 3: Task completed" > data/test.txt
    aws --endpoint-url=http://localhost:4566 s3 cp data/test.txt s3://$BUCKET_NAME/logs/test.txt
    echo "✅ TXT file uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.text_logs.find().pretty()\""
}

test_xml() {
    echo "📄 Creating XML test file..."
    mkdir -p data
    echo '<?xml version="1.0" encoding="UTF-8"?>
<data>
  <record id="1">
    <name>John Doe</name>
    <age>30</age>
    <email>john.doe@example.com</email>
  </record>
  <record id="2">
    <name>Jane Smith</name>
    <age>25</age>
    <email>jane.smith@example.com</email>
  </record>
</data>' > data/test.xml
    aws --endpoint-url=http://localhost:4566 s3 cp data/test.xml s3://$BUCKET_NAME/data/test.xml
    echo "✅ XML file uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.xml_data.find().pretty()\""
}

test_xls() {
    echo "📄 Creating XLS test file..."
    mkdir -p data
    echo "name,age,department
Alice,28,HR
Charlie,32,Finance" > data/test.csv
    # Convert to xlsx using a simple approach (rename for testing)
    cp data/test.csv data/test.xlsx
    aws --endpoint-url=http://localhost:4566 s3 cp data/test.xlsx s3://$BUCKET_NAME/data/test.xlsx
    echo "✅ XLS file uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.xls_data.find().pretty()\""
}

test_pdf() {
    echo "📄 Creating PDF test file..."
    mkdir -p data
    echo "%PDF-1.4
1 0 obj
<<
/Type /Catalog
/Pages 2 0 R
>>
endobj

2 0 obj
<<
/Type /Pages
/Kids [3 0 R]
/Count 1
>>
endobj

3 0 obj
<<
/Type /Page
/Parent 2 0 R
/MediaBox [0 0 612 792]
/Contents 4 0 R
>>
endobj

4 0 obj
<<
/Length 44
>>
stream
BT
/F1 12 Tf
72 720 Td
(Test PDF Document) Tj
ET
endstream
endobj

xref
0 5
0000000000 65535 f 
0000000009 00000 n 
0000000074 00000 n 
0000000120 00000 n 
0000000179 00000 n 
trailer
<<
/Size 5
/Root 1 0 R
>>
startxref
238
%%EOF" > data/test.pdf
    aws --endpoint-url=http://localhost:4566 s3 cp data/test.pdf s3://$BUCKET_NAME/documents/test.pdf
    echo "✅ PDF file uploaded! Verify with: docker-compose exec mongodb mongosh ingestion_db --eval \"db.pdf_documents.find().pretty()\""
}

case $FILE_TYPE in
    "csv")
        test_csv
        ;;
    "txt")
        test_txt
        ;;
    "json")
        test_json
        ;;
    "xml")
        test_xml
        ;;
    "xls")
        test_xls
        ;;
    "pdf")
        test_pdf
        ;;
    "all")
        test_csv
        test_json
        test_txt
        test_xml
        test_xls
        test_pdf
        ;;
esac

echo "✅ Test completed for $FILE_TYPE!"