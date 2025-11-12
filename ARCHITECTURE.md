# adi-svc Architecture

## Overview

adi-svc is a production-grade Rust microservice that wraps Azure AI Document Intelligence, following Domain-Driven Design (DDD) and Hexagonal Architecture (Ports and Adapters) principles.

## Architecture Layers

### 1. Domain Layer (`src/domain/`)

The innermost layer containing pure business logic with no external dependencies.

**Components:**
- `models.rs` - Core domain entities (AnalysisOperation, AnalysisResult, DocumentPage, etc.)
- `value_objects.rs` - Immutable value objects (ModelType, DocumentSource, Locale, etc.)
- `errors.rs` - Domain-specific errors

**Key Principles:**
- No dependencies on external frameworks
- Pure Rust types
- Business rule validation
- Framework-agnostic

### 2. Application Layer (`src/application/`)

Orchestrates domain objects and coordinates with adapters to fulfill use cases.

**Components:**
- `services.rs` - Application services (DocumentIntelligenceService)
- `ports.rs` - Port trait definitions (interfaces for adapters)
- `errors.rs` - Application-level errors

**Key Services:**
- `DocumentIntelligenceService` - Main orchestrator for document analysis

**Ports (Interfaces):**
- `DocumentIntelligencePort` - Interface for document analysis operations
- `DocumentStoragePort` - Interface for document storage
- `OperationTrackerPort` - Interface for operation tracking

### 3. Infrastructure Layer (`src/infrastructure/`)

Implements the ports using concrete technologies (Azure, file system, memory).

**Adapters:**
- `azure.rs` - Azure AI Document Intelligence REST API adapter
- `storage.rs` - Local file storage adapter
- `tracker.rs` - In-memory operation tracker
- `config.rs` - Configuration management

**Azure Adapter:**
- Uses reqwest for HTTP communication
- Implements Azure REST API v4.0 (2024-02-29-preview)
- Handles authentication via API key
- Converts between Azure DTOs and domain models

### 4. Presentation Layer (`src/presentation/`)

Exposes the service via gRPC and REST APIs.

**Components:**
- `grpc.rs` - gRPC server implementation (tonic)
- `rest.rs` - REST API implementation (axum)
- `converters.rs` - Protobuf ↔ Domain model conversion

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        Client Request                        │
│                    (gRPC or REST)                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│              (gRPC Server / REST API)                        │
│  • Validate request                                          │
│  • Convert to domain models                                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│           (DocumentIntelligenceService)                      │
│  • Validate business rules                                   │
│  • Orchestrate operations                                    │
│  • Call ports (interfaces)                                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                       │
│                  (Concrete Adapters)                         │
│  • Azure AI Document Intelligence Adapter                    │
│  • Local File Storage Adapter                                │
│  • In-Memory Operation Tracker                               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    External Services                         │
│        (Azure AI Document Intelligence API)                  │
└─────────────────────────────────────────────────────────────┘
```

## Protocol Buffers

All gRPC services are defined in `proto/document_intelligence.proto`:

- `DocumentIntelligenceService` - Main service interface
- Comprehensive message types for all Azure Document Intelligence models
- Support for streaming file uploads

Generated code is placed in `src/generated/` via `tonic-build`.

## Dependency Injection

The service uses constructor-based dependency injection:

```rust
DocumentIntelligenceService::new(
    intelligence_adapter: Arc<dyn DocumentIntelligencePort>,
    storage_adapter: Option<Arc<dyn DocumentStoragePort>>,
    tracker_adapter: Option<Arc<dyn OperationTrackerPort>>,
)
```

This allows:
- Easy testing with mock adapters
- Flexible adapter implementations
- Clear dependency relationships

## Configuration

Configuration is loaded from environment variables via `Config::from_env()`:

- `AZURE_DOCUMENT_INTELLIGENCE_ENDPOINT` - Azure endpoint URL
- `AZURE_DOCUMENT_INTELLIGENCE_KEY` - Azure API key
- `GRPC_PORT` - gRPC server port (default: 50051)
- `REST_PORT` - REST API port (default: 8080)
- `UPLOAD_DIR` - Directory for uploaded files
- `MAX_UPLOAD_SIZE_MB` - Maximum upload size

## Error Handling

Three-tier error system:

1. **Domain Errors** (`DomainError`) - Business rule violations
2. **Application Errors** (`ApplicationError`) - Use case failures
3. **Presentation Errors** - HTTP status codes / gRPC status

Errors are converted at layer boundaries:
- Domain → Application: `From<DomainError> for ApplicationError`
- Application → Presentation: Custom error handlers

## Testing Strategy

### Unit Tests
- Domain logic tests (value objects, models)
- Application service tests with mock adapters
- Adapter tests with mock HTTP clients

### Integration Tests
- End-to-end gRPC tests
- End-to-end REST tests
- Azure adapter tests (with real/mock Azure service)

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Test implementations
}
```

## Deployment

### Docker
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/adi-svc /usr/local/bin/
CMD ["adi-svc"]
```

### Kubernetes
- StatefulSet for service instances
- ConfigMap for configuration
- Secret for Azure credentials
- Service for gRPC and REST endpoints

## Monitoring

### Logging
- Structured logging via `tracing`
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- JSON output for production

### Metrics (Future)
- Request count
- Request duration
- Error rates
- Azure API latency

### Health Checks
- REST: `GET /health`
- gRPC: Health check service (to be implemented)

## Security

- API keys stored in environment variables
- HTTPS for all Azure communications
- Input validation at domain layer
- Rate limiting (to be implemented)
- Authentication/Authorization (to be implemented)

## Performance Considerations

- Async I/O throughout (tokio)
- Connection pooling (reqwest)
- Streaming for large file uploads
- Efficient protobuf serialization
- Optional caching for results

## Future Enhancements

1. **Database Integration** - Replace in-memory tracker with PostgreSQL/Redis
2. **Distributed Tracing** - OpenTelemetry integration
3. **Metrics** - Prometheus metrics
4. **Authentication** - JWT/OAuth2 support
5. **Rate Limiting** - Per-client rate limits
6. **Caching** - Redis cache for results
7. **Retry Logic** - Automatic retry for transient failures
8. **Circuit Breaker** - Protect against Azure API failures
9. **WebSocket Support** - Real-time progress updates
10. **Custom Model Training** - API for training custom models

