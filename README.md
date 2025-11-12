# adi-svc: Azure AI Document Intelligence Service

A production-grade Rust microservice wrapping Azure AI Document Intelligence with gRPC and REST endpoints.

## Architecture

This service implements **Domain-Driven Design (DDD)** with **Hexagonal Architecture (Ports and Adapters)**:

```
┌─────────────────────────────────────────────────────────────┐
│                        Presentation Layer                    │
│                    (gRPC Server / REST API)                  │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                      Application Layer                       │
│                    (Use Cases / Services)                    │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│                        Domain Layer                          │
│                  (Business Logic / Entities)                 │
└──────────────────────────────────────────────────────────────┘
                        ▲
┌───────────────────────┴─────────────────────────────────────┐
│                    Infrastructure Layer                      │
│         (Azure AI Document Intelligence Adapter)             │
└──────────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Read API** - Extract text from documents
- ✅ **Layout API** - Extract text, tables, and document structure
- ✅ **Prebuilt Models** - Invoice, Receipt, ID, Business Card, W-2, etc.
- ✅ **Custom Models** - Train and use custom extraction models
- ✅ **Dual Endpoints** - Both gRPC and REST APIs
- ✅ **Document Upload** - Direct file upload support
- ✅ **Async Processing** - Non-blocking document processing
- ✅ **Production Ready** - Proper error handling, logging, and monitoring

## Project Structure

```
adi/
├── adi-svc/                    # Main Rust service
│   ├── proto/                  # Protobuf definitions
│   ├── src/
│   │   ├── domain/            # Domain entities and logic
│   │   ├── application/       # Use cases and services
│   │   ├── infrastructure/    # Azure adapter implementation
│   │   ├── presentation/      # gRPC and REST servers
│   │   └── main.rs
│   ├── Cargo.toml
│   └── build.rs
├── adi-web/                    # React testing frontend
│   ├── src/
│   ├── public/
│   └── package.json
└── README.md
```

## Azure AI Document Intelligence Models

### 1. Read Model
Extracts text lines and words from documents and images.

### 2. Layout Model
Extracts text, tables, selection marks, and document structure.

### 3. Prebuilt Models
- **Invoice**: Extract key fields from invoices
- **Receipt**: Extract transaction details from receipts
- **ID Document**: Extract information from passports, driver's licenses
- **Business Card**: Extract contact information
- **W-2**: Extract tax form data
- **Health Insurance Card**: Extract insurance information

### 4. Custom Models
Train models on your own documents for custom field extraction.

## Prerequisites

- Rust 1.70+
- Node.js 18+ (for React frontend)
- Azure subscription with Document Intelligence resource
- Azure Document Intelligence API key and endpoint

## Setup

### 1. Clone and Build

```bash
cd adi
cargo build --release
```

### 2. Configure Azure Credentials

Create `.env` file in the `adi-svc` directory:

```env
AZURE_DOCUMENT_INTELLIGENCE_ENDPOINT=https://your-resource.cognitiveservices.azure.com/
AZURE_DOCUMENT_INTELLIGENCE_KEY=your-api-key
```

### 3. Run the Service

```bash
# Start gRPC server (port 50051) and REST API (port 8080)
cargo run --release
```

### 4. Run the React Frontend

```bash
cd adi-web
npm install
npm start
```

Visit http://localhost:3000 to access the testing UI.

## API Documentation

### gRPC API

See [proto/document_intelligence.proto](adi-svc/proto/document_intelligence.proto) for full API definition.

### REST API

Base URL: `http://localhost:8080`

#### Analyze Document with Read Model
```bash
POST /api/v1/analyze/read
Content-Type: application/json

{
  "document_url": "https://example.com/document.pdf"
}
```

#### Analyze Document with Layout Model
```bash
POST /api/v1/analyze/layout
Content-Type: application/json

{
  "document_url": "https://example.com/document.pdf"
}
```

#### Upload and Analyze Document
```bash
POST /api/v1/upload/read
Content-Type: multipart/form-data

file: <binary data>
```

## Development

### Run Tests
```bash
cargo test
```

### Run with Logging
```bash
RUST_LOG=debug cargo run
```

### Generate Protobuf Code
```bash
cargo build  # Automatically runs build.rs
```

## Dependencies

### Core Dependencies
- **tonic** - gRPC framework
- **prost** - Protocol Buffers implementation
- **tokio** - Async runtime
- **azure_core** - Azure SDK core
- **azure_identity** - Azure authentication
- **reqwest** - HTTP client for Azure REST API

### Additional Dependencies
- **axum** - REST API framework
- **tower** - Service middleware
- **tracing** - Structured logging
- **serde** - Serialization/deserialization
- **anyhow** - Error handling

## Testing

The service includes:
- Unit tests for domain logic
- Integration tests for Azure adapter
- End-to-end tests for gRPC and REST APIs
- React UI for manual testing

## Security

- API keys stored in environment variables
- HTTPS for all Azure communications
- Input validation and sanitization
- Rate limiting (configurable)

## License

Apache 2.0

## References

- [Azure AI Document Intelligence Documentation](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/)
- [Azure Rust SDK](https://github.com/Azure/azure-sdk-for-rust)
- [tonic gRPC Framework](https://github.com/hyperium/tonic)

