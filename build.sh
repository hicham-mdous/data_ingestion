#!/bin/bash

# Manual build and push to ECR
# Usage: ./build.sh <aws-account-id> <region>

if [ $# -ne 2 ]; then
    echo "Usage: $0 <aws-account-id> <region>"
    echo "Example: $0 123456789012 us-east-1"
    exit 1
fi

ACCOUNT_ID=$1
REGION=$2
REPO_NAME="data-ingestion"
IMAGE_TAG="latest"

echo "🔨 Building Docker image..."
docker build -t $REPO_NAME:$IMAGE_TAG .

echo "🔐 Logging into ECR..."
aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin $ACCOUNT_ID.dkr.ecr.$REGION.amazonaws.com

echo "🏷️ Tagging image..."
docker tag $REPO_NAME:$IMAGE_TAG $ACCOUNT_ID.dkr.ecr.$REGION.amazonaws.com/$REPO_NAME:$IMAGE_TAG

echo "📤 Pushing to ECR..."
docker push $ACCOUNT_ID.dkr.ecr.$REGION.amazonaws.com/$REPO_NAME:$IMAGE_TAG

echo "✅ Image pushed: $ACCOUNT_ID.dkr.ecr.$REGION.amazonaws.com/$REPO_NAME:$IMAGE_TAG"