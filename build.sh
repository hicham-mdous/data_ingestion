#!/bin/bash

# Build for ECS
cargo build --release

# Build Docker image
docker build -t data-ingestion-ecs .

echo "ECS deployment ready. Push to ECR:"
echo "aws ecr get-login-password --region <region> | docker login --username AWS --password-stdin <account>.dkr.ecr.<region>.amazonaws.com"
echo "docker tag data-ingestion-ecs:latest <account>.dkr.ecr.<region>.amazonaws.com/data-ingestion-ecs:latest"
echo "docker push <account>.dkr.ecr.<region>.amazonaws.com/data-ingestion-ecs:latest"