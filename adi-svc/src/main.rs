/// adi-svc main entry point
/// 
/// This starts both the gRPC and REST servers.

use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use adi_svc::application::services::DocumentIntelligenceService;
use adi_svc::infrastructure::{
    AzureDocumentIntelligenceAdapter, Config, PostgresOperationTracker,
    LocalFileStorageAdapter,
};
use adi_svc::presentation::{GrpcDocumentIntelligenceService, create_rest_router};
use adi_svc::generated::document_intelligence_service_server::DocumentIntelligenceServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "adi_svc=debug,tower_http=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting adi-svc...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");
    info!("Azure endpoint: {}", config.azure.endpoint);
    info!("gRPC server will listen on {}:{}", config.server.host, config.server.grpc_port);
    info!("REST server will listen on {}:{}", config.server.host, config.server.rest_port);

    // Initialize adapters
    let azure_adapter = Arc::new(AzureDocumentIntelligenceAdapter::new(config.azure.clone()));
    let storage_adapter = Arc::new(LocalFileStorageAdapter::new(config.storage.clone()).await?);
    
    // Initialize PostgreSQL tracker
    info!("Connecting to PostgreSQL database...");
    let tracker_adapter = Arc::new(
        PostgresOperationTracker::new(&config.database.url).await?
    );

    // Initialize application service
    let app_service = Arc::new(DocumentIntelligenceService::new(
        azure_adapter,
        Some(storage_adapter),
        Some(tracker_adapter),
    ));

    // Clone for REST server
    let app_service_rest = app_service.clone();

    // Start gRPC server
    let grpc_addr: std::net::SocketAddr = format!("{}:{}", config.server.host, config.server.grpc_port).parse()?;
    let grpc_service = GrpcDocumentIntelligenceService::new(app_service);
    
    info!("Starting gRPC server on {}", grpc_addr);
    let grpc_server = async move {
        if let Err(e) = Server::builder()
            .add_service(DocumentIntelligenceServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
        {
            error!("gRPC server error: {}", e);
        }
    };

    // Start REST server
    let rest_addr: std::net::SocketAddr = format!("{}:{}", config.server.host, config.server.rest_port).parse()?;
    let rest_router = create_rest_router(app_service_rest);
    
    info!("Starting REST server on {}", rest_addr);
    let rest_server = async move {
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        if let Err(e) = axum::serve(listener, rest_router).await {
            error!("REST server error: {}", e);
        }
    };

    // Run both servers concurrently
    info!("adi-svc is running!");
    info!("gRPC endpoint: {}:{}", config.server.host, config.server.grpc_port);
    info!("REST endpoint: http://{}:{}", config.server.host, config.server.rest_port);
    info!("Health check: http://{}:{}/health", config.server.host, config.server.rest_port);
    
    tokio::select! {
        _ = grpc_server => {
            error!("gRPC server stopped unexpectedly");
        }
        _ = rest_server => {
            error!("REST server stopped unexpectedly");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    info!("Shutting down adi-svc...");
    Ok(())
}

