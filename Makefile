.PHONY: help build test run clean run-all setup db-up db-down db-migrate backend frontend docker-build docker-up docker-down install check fmt lint

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

## help: Show this help message
help:
	@echo "$(BLUE)adi-svc Makefile Commands$(NC)"
	@echo ""
	@echo "$(GREEN)Setup & Configuration:$(NC)"
	@echo "  make setup          - Initial project setup (copy .env, install deps)"
	@echo "  make install        - Install Rust and Node.js dependencies"
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@echo "  make build          - Build Rust backend"
	@echo "  make test           - Run all tests"
	@echo "  make check          - Check Rust code without building"
	@echo "  make fmt            - Format Rust code"
	@echo "  make lint           - Run clippy linter"
	@echo ""
	@echo "$(GREEN)Database:$(NC)"
	@echo "  make db-up          - Start PostgreSQL database"
	@echo "  make db-down        - Stop PostgreSQL database"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Reset database (drop and recreate)"
	@echo ""
	@echo "$(GREEN)Running Services:$(NC)"
	@echo "  make backend        - Run Rust backend service"
	@echo "  make frontend       - Run React frontend (dev mode)"
	@echo "  make run            - Run backend and frontend concurrently"
	@echo "  make run-all        - Start everything (DB + Backend + Frontend)"
	@echo ""
	@echo "$(GREEN)Docker:$(NC)"
	@echo "  make docker-build   - Build Docker images"
	@echo "  make docker-up      - Start all services with Docker Compose"
	@echo "  make docker-down    - Stop Docker Compose services"
	@echo "  make docker-logs    - View Docker logs"
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make clean-all      - Clean everything (artifacts + Docker volumes)"
	@echo ""

## setup: Initial project setup
setup:
	@echo "$(BLUE)Setting up adi-svc project...$(NC)"
	@if [ ! -f adi-svc/.env ]; then \
		echo "$(YELLOW)Creating .env file from .env.example$(NC)"; \
		cp adi-svc/.env.example adi-svc/.env; \
		echo "$(RED)⚠️  Please edit adi-svc/.env with your Azure credentials$(NC)"; \
	else \
		echo "$(GREEN)✓ .env file already exists$(NC)"; \
	fi
	@echo "$(GREEN)Setup complete! Run 'make install' to install dependencies.$(NC)"

## install: Install all dependencies
install:
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@echo "$(YELLOW)Checking Rust installation...$(NC)"
	@command -v cargo >/dev/null 2>&1 || { echo "$(RED)Rust not found. Install from https://rustup.rs/$(NC)"; exit 1; }
	@echo "$(GREEN)✓ Rust is installed$(NC)"
	@echo "$(YELLOW)Checking Node.js installation...$(NC)"
	@command -v node >/dev/null 2>&1 || { echo "$(RED)Node.js not found. Install from https://nodejs.org/$(NC)"; exit 1; }
	@echo "$(GREEN)✓ Node.js is installed$(NC)"
	@echo "$(BLUE)Installing Node.js dependencies...$(NC)"
	cd adi-web && npm install
	@echo "$(GREEN)✓ All dependencies installed$(NC)"

## build: Build the Rust backend
build:
	@echo "$(BLUE)Building adi-svc backend...$(NC)"
	cd adi-svc && cargo build --release
	@echo "$(GREEN)✓ Build complete$(NC)"

## test: Run all tests
test:
	@echo "$(BLUE)Running Rust tests...$(NC)"
	cd adi-svc && cargo test
	@echo "$(GREEN)✓ Tests complete$(NC)"

## check: Check Rust code without building
check:
	@echo "$(BLUE)Checking Rust code...$(NC)"
	cd adi-svc && cargo check
	@echo "$(GREEN)✓ Check complete$(NC)"

## fmt: Format Rust code
fmt:
	@echo "$(BLUE)Formatting Rust code...$(NC)"
	cd adi-svc && cargo fmt
	@echo "$(GREEN)✓ Formatting complete$(NC)"

## lint: Run clippy linter
lint:
	@echo "$(BLUE)Running clippy...$(NC)"
	cd adi-svc && cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Linting complete$(NC)"

## db-up: Start PostgreSQL database
db-up:
	@echo "$(BLUE)Starting PostgreSQL database...$(NC)"
	docker-compose up -d postgres
	@echo "$(YELLOW)Waiting for database to be ready...$(NC)"
	@sleep 3
	@echo "$(GREEN)✓ Database is running$(NC)"

## db-down: Stop PostgreSQL database
db-down:
	@echo "$(BLUE)Stopping PostgreSQL database...$(NC)"
	docker-compose stop postgres
	@echo "$(GREEN)✓ Database stopped$(NC)"

## db-migrate: Run database migrations
db-migrate: db-up
	@echo "$(BLUE)Running database migrations...$(NC)"
	cd adi-svc && cargo run --bin migrate
	@echo "$(GREEN)✓ Migrations complete$(NC)"

## db-reset: Reset database
db-reset:
	@echo "$(RED)Resetting database...$(NC)"
	docker-compose down -v postgres
	@$(MAKE) db-up
	@$(MAKE) db-migrate
	@echo "$(GREEN)✓ Database reset complete$(NC)"

## backend: Run the Rust backend service
backend: db-up db-migrate
	@echo "$(BLUE)Starting adi-svc backend...$(NC)"
	@echo "$(YELLOW)Backend running on:$(NC)"
	@echo "  gRPC: http://localhost:50051"
	@echo "  REST: http://localhost:8080"
	@echo "  Health: http://localhost:8080/health"
	cd adi-svc && cargo run --release --bin adi-svc

## frontend: Run the React frontend
frontend:
	@echo "$(BLUE)Starting React frontend...$(NC)"
	@echo "$(YELLOW)Frontend running on: http://localhost:3000$(NC)"
	cd adi-web && npm start

## run: Run backend and frontend concurrently
run:
	@echo "$(BLUE)Starting all services...$(NC)"
	@trap 'kill 0' EXIT; \
	$(MAKE) db-up && \
	(cd adi-svc && cargo run --release --bin adi-svc & echo $$! > /tmp/adi-backend.pid) & \
	sleep 5 && \
	(cd adi-web && npm start & echo $$! > /tmp/adi-frontend.pid) & \
	wait

## run-all: Start everything (Database + Backend + Frontend)
run-all: setup install db-up db-migrate
	@echo "$(GREEN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(GREEN)  🚀 Starting adi-svc - Full Stack$(NC)"
	@echo "$(GREEN)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo ""
	@echo "$(BLUE)✓ Database:$(NC) PostgreSQL running on localhost:5432"
	@echo "$(BLUE)Starting Backend...$(NC)"
	@echo ""
	@trap 'echo "\n$(BLUE)Shutting down services...$(NC)"; kill 0; $(MAKE) db-down' EXIT INT TERM; \
	(cd adi-svc && cargo run --release --bin adi-svc 2>&1) & \
	BACKEND_PID=$$!; \
	echo "$(YELLOW)Waiting for backend to be ready...$(NC)"; \
	for i in 1 2 3 4 5 6 7 8 9 10; do \
		if curl -s http://localhost:8080/health > /dev/null 2>&1; then \
			echo "$(GREEN)✓ Backend is ready!$(NC)"; \
			echo "$(BLUE)✓ Backend gRPC:$(NC) Running on localhost:50051"; \
			echo "$(BLUE)✓ Backend REST:$(NC) Running on http://localhost:8080"; \
			break; \
		fi; \
		echo "Waiting... ($$i/10)"; \
		sleep 2; \
	done; \
	echo "$(BLUE)✓ Frontend UI:$(NC) Starting on http://localhost:3000"; \
	echo ""; \
	echo "$(YELLOW)Press Ctrl+C to stop all services$(NC)"; \
	echo ""; \
	(cd adi-web && npm start) & \
	wait

## docker-build: Build Docker images
docker-build:
	@echo "$(BLUE)Building Docker images...$(NC)"
	docker-compose build
	@echo "$(GREEN)✓ Docker images built$(NC)"

## docker-up: Start all services with Docker Compose
docker-up:
	@echo "$(BLUE)Starting all services with Docker Compose...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)✓ All services started$(NC)"
	@echo ""
	@echo "$(YELLOW)Services:$(NC)"
	@echo "  Database: localhost:5432"
	@echo "  gRPC: localhost:50051"
	@echo "  REST API: http://localhost:8080"
	@echo "  Health: http://localhost:8080/health"
	@echo ""
	@echo "View logs: make docker-logs"

## docker-down: Stop Docker Compose services
docker-down:
	@echo "$(BLUE)Stopping Docker Compose services...$(NC)"
	docker-compose down
	@echo "$(GREEN)✓ Services stopped$(NC)"

## docker-logs: View Docker logs
docker-logs:
	docker-compose logs -f

## clean: Clean build artifacts
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cd adi-svc && cargo clean
	cd adi-web && rm -rf build node_modules/.cache
	@echo "$(GREEN)✓ Clean complete$(NC)"

## clean-all: Clean everything including Docker volumes
clean-all: clean
	@echo "$(BLUE)Cleaning Docker volumes...$(NC)"
	docker-compose down -v
	cd adi-web && rm -rf node_modules
	@echo "$(GREEN)✓ Full clean complete$(NC)"

# Development helpers
.PHONY: dev-backend dev-frontend dev-db

## dev-backend: Run backend in development mode with hot reload
dev-backend: db-up
	@echo "$(BLUE)Running backend in development mode...$(NC)"
	cd adi-svc && cargo watch -x "run --bin adi-svc"

## dev-frontend: Run frontend in development mode
dev-frontend:
	@echo "$(BLUE)Running frontend in development mode...$(NC)"
	cd adi-web && npm start

## dev-db: Open psql console
dev-db:
	@echo "$(BLUE)Opening PostgreSQL console...$(NC)"
	docker-compose exec postgres psql -U adi_user -d adi_db

