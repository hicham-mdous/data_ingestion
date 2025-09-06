#!/bin/bash

# Configuration
BUCKET_NAME=${BUCKET_NAME:-"data-ingestion-bucket"}
QUEUE_NAME=${QUEUE_NAME:-"test-queue"}

# Create S3 bucket
echo "Creating S3 bucket: $BUCKET_NAME"
awslocal s3 mb s3://$BUCKET_NAME

# Create SQS queue
echo "Creating SQS queue: $QUEUE_NAME"
awslocal sqs create-queue --queue-name $QUEUE_NAME

# Configure S3 bucket notification to SQS
echo "Configuring S3 notification to SQS"
awslocal s3api put-bucket-notification-configuration \
  --bucket $BUCKET_NAME \
  --notification-configuration "{
    \"QueueConfigurations\": [{
      \"Id\": \"s3-sqs-notification\",
      \"QueueArn\": \"arn:aws:sqs:us-east-1:000000000000:$QUEUE_NAME\",
      \"Events\": [\"s3:ObjectCreated:*\"]
    }]
  }"