#!/bin/bash

set -e

# 部署脚本
# 用法: ./scripts/deploy.sh [environment]

ENVIRONMENT=${1:-staging}
VERSION=$(git describe --tags --always)

echo "Deploying NFA Core version $VERSION to $ENVIRONMENT environment..."

# 检查所需工具
command -v docker >/dev/null 2>&1 || { echo "Docker is required"; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "Docker Compose is required"; exit 1; }

# 构建 Docker 镜像
echo "Building Docker images..."
docker-compose build

# 标签镜像
echo "Tagging images with version $VERSION..."
docker tag nfa-broker:latest nfa-broker:$VERSION
docker tag nfa-runtime:latest nfa-runtime:$VERSION

# 根据环境选择部署配置
case $ENVIRONMENT in
    staging)
        DEPLOY_COMPOSE="docker-compose.staging.yml"
        ;;
    production)
        DEPLOY_COMPOSE="docker-compose.production.yml"
        ;;
    *)
        DEPLOY_COMPOSE="docker-compose.yml"
        ;;
esac

# 部署服务
echo "Deploying services using $DEPLOY_COMPOSE..."
docker-compose -f $DEPLOY_COMPOSE up -d

# 等待服务启动
echo "Waiting for services to start..."
sleep 10

# 运行健康检查
echo "Running health checks..."
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "Deployment successful!"
else
    echo "Deployment failed: health check failed"
    exit 1
fi

# 记录部署
echo "Logging deployment..."
echo "$(date): Deployed version $VERSION to $ENVIRONMENT" >> deploy.log

echo "Deployment completed successfully!"