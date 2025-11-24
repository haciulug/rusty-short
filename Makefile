.PHONY: help build run test clean docker-build docker-run docker-stop k8s-deploy k8s-delete test-rate-limit

help:
	@echo "RustyShort - Makefile commands"
	@echo ""
	@echo "  make build             - Build the project"
	@echo "  make run               - Run the application"
	@echo "  make test              - Run tests"
	@echo "  make clean             - Clean build artifacts"
	@echo "  make docker-build      - Build Docker image"
	@echo "  make docker-run        - Run with docker-compose"
	@echo "  make docker-stop       - Stop docker-compose"

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

docker-build:
	docker build -t rustyshort:latest .

docker-run:
	docker-compose up -d

docker-stop:
	docker-compose down
