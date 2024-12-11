.PHONY: build deploy clean all

# 默认目标
all: build deploy

# 构建 Rust Lambda 函数
build:
	@echo "Building Rust Lambda function..."
	cd aws_axum_lambda && cargo lambda build --release  --target aarch64-unknown-linux-musl   --output-format zip
	@echo "Build completed!"

# 部署到 AWS
deploy:
	@echo "Deploying to AWS..."
	serverless deploy
	@echo "Deployment completed!"

# 清理构建文件
clean:
	@echo "Cleaning build files..."
	cd aws_axum_lambda && cargo clean
	# rm -rf .serverless
	@echo "Clean completed!"

# 构建并部署
build-deploy: build deploy

# 显示帮助信息
help:
	@echo "Available commands:"
	@echo "  make build       - Build the Rust Lambda function"
	@echo "  make deploy      - Deploy to AWS"
	@echo "  make clean       - Clean build files"
	@echo "  make all         - Build and deploy (default)"
	@echo "  make build-deploy- Same as 'make all'"
	@echo "  make help        - Show this help message"
