#!/bin/bash

# Create S3 bucket
awslocal s3 mb s3://test-bucket

# Create SQS queue
awslocal sqs create-queue --queue-name test-queue

# Configure S3 bucket notification to SQS
awslocal s3api put-bucket-notification-configuration \
  --bucket test-bucket \
  --notification-configuration '{
    "QueueConfigurations": [{
      "Id": "s3-sqs-notification",
      "QueueArn": "arn:aws:sqs:us-east-1:000000000000:test-queue",
      "Events": ["s3:ObjectCreated:*"]
    }]
  }'