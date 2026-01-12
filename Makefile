.PHONY: all build run test clean docker web init help

# Variables
BINARY_NAME=roead
BINARY_PATH=./cmd/roead
GO=go
DOCKER=docker
NPM=npm

# Build information
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
BUILD_TIME=$(shell date -u '+%Y-%m-%d_%H:%M:%S')
LDFLAGS=-ldflags "-X main.version=$(VERSION) -X main.buildTime=$(BUILD_TIME)"

# Default target
all: build

## help: Show this help message
help:
	@echo "Roea AI - Makefile targets:"
	@echo ""
	@echo "  make build       Build the roead binary"
	@echo "  make run         Build and run the server"
	@echo "  make test        Run tests"
	@echo "  make clean       Clean build artifacts"
	@echo "  make docker      Build Docker images"
	@echo "  make web         Build the web frontend"
	@echo "  make init        Initialize a new Roea instance"
	@echo "  make dev         Run in development mode"
	@echo "  make lint        Run linters"
	@echo ""

## build: Build the binary
build:
	$(GO) build $(LDFLAGS) -o $(BINARY_NAME) $(BINARY_PATH)

## run: Build and run the server
run: build
	./$(BINARY_NAME)

## test: Run tests
test:
	$(GO) test -v ./...

## test-coverage: Run tests with coverage
test-coverage:
	$(GO) test -v -coverprofile=coverage.out ./...
	$(GO) tool cover -html=coverage.out -o coverage.html

## clean: Clean build artifacts
clean:
	rm -f $(BINARY_NAME)
	rm -f coverage.out coverage.html
	rm -rf .roea/
	rm -f roea.fossil

## docker: Build Docker images
docker:
	$(DOCKER) build -f deploy/docker/Dockerfile.roead -t roea-ai/roead:$(VERSION) .
	$(DOCKER) build -f deploy/docker/Dockerfile.agent-runtime -t roea-ai/agent-runtime:$(VERSION) .

## docker-push: Push Docker images
docker-push: docker
	$(DOCKER) push roea-ai/roead:$(VERSION)
	$(DOCKER) push roea-ai/agent-runtime:$(VERSION)

## web: Build the web frontend
web:
	cd web && $(NPM) install && $(NPM) run build

## web-dev: Run the web frontend in development mode
web-dev:
	cd web && $(NPM) install && $(NPM) run dev

## init: Initialize a new Roea instance
init: build
	./$(BINARY_NAME) --init --path .

## dev: Run in development mode (server + web)
dev:
	@echo "Starting Roea in development mode..."
	@echo "Starting server..."
	./$(BINARY_NAME) &
	@sleep 2
	@echo "Starting web frontend..."
	cd web && $(NPM) run dev

## lint: Run linters
lint:
	golangci-lint run ./...

## fmt: Format Go code
fmt:
	$(GO) fmt ./...

## mod-tidy: Tidy Go modules
mod-tidy:
	$(GO) mod tidy

## deps: Download dependencies
deps:
	$(GO) mod download
	cd web && $(NPM) install

## k8s-apply: Apply Kubernetes configs
k8s-apply:
	kubectl apply -k deploy/k8s/base/

## k8s-delete: Delete Kubernetes resources
k8s-delete:
	kubectl delete -k deploy/k8s/base/

## compose-up: Start with docker-compose
compose-up:
	cd deploy/docker && $(DOCKER) compose up -d

## compose-down: Stop docker-compose
compose-down:
	cd deploy/docker && $(DOCKER) compose down

## compose-logs: Show docker-compose logs
compose-logs:
	cd deploy/docker && $(DOCKER) compose logs -f
