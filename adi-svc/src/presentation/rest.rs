/// REST API implementation using Axum
/// 
/// This module provides a RESTful HTTP API for document analysis.

use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

use crate::application::services::DocumentIntelligenceService;
use crate::domain::*;

/// REST API state
#[derive(Clone)]
pub struct RestApiState {
    pub service: Arc<DocumentIntelligenceService>,
}

/// Create REST API router
pub fn create_rest_router(service: Arc<DocumentIntelligenceService>) -> Router {
    let state = RestApiState { service };
    
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Analysis endpoints
        .route("/api/v1/analyze/read", post(analyze_read))
        .route("/api/v1/analyze/layout", post(analyze_layout))
        .route("/api/v1/analyze/invoice", post(analyze_invoice))
        .route("/api/v1/analyze/receipt", post(analyze_receipt))
        .route("/api/v1/analyze/id-document", post(analyze_id_document))
        .route("/api/v1/analyze/business-card", post(analyze_business_card))
        .route("/api/v1/analyze/w2", post(analyze_w2))
        .route("/api/v1/analyze/custom/:model_id", post(analyze_custom))
        
        // Upload endpoints
        .route("/api/v1/upload/read", post(upload_and_analyze_read))
        .route("/api/v1/upload/layout", post(upload_and_analyze_layout))
        .route("/api/v1/upload/invoice", post(upload_and_analyze_invoice))
        
        // Results endpoint
        .route("/api/v1/results/:operation_id", get(get_result))
        
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}

// DTOs for REST API
#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeUrlRequest {
    document_url: String,
    #[serde(default)]
    options: RestAnalyzeOptions,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RestAnalyzeOptions {
    locale: Option<String>,
    pages: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct AnalyzeResponse {
    operation_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<RestAnalysisResult>,
}

#[derive(Debug, Serialize)]
struct RestAnalysisResult {
    model_id: String,
    content: String,
    pages: Vec<RestPage>,
    tables: Vec<RestTable>,
}

#[derive(Debug, Serialize)]
struct RestPage {
    page_number: i32,
    width: f32,
    height: f32,
    word_count: usize,
    line_count: usize,
}

#[derive(Debug, Serialize)]
struct RestTable {
    row_count: i32,
    column_count: i32,
    cell_count: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Handler implementations
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "adi-svc",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn analyze_read(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze read request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::Read)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_layout(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze layout request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::Layout)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_invoice(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze invoice request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::Invoice)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_receipt(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
    ) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze receipt request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::Receipt)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_id_document(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze ID document request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::IdDocument)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_business_card(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze business card request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::BusinessCard)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_w2(
    State(state): State<RestApiState>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze W-2 request for: {}", request.document_url);
    
    let domain_request = create_domain_request(request, ModelType::W2)?;
    let operation = state.service.analyze_document(domain_request).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn analyze_custom(
    State(state): State<RestApiState>,
    Path(model_id): Path<String>,
    Json(request): Json<AnalyzeUrlRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Analyze custom request with model: {}", model_id);
    
    let source = DocumentSource::Url(request.document_url);
    let operation = state.service.analyze_custom(source, &model_id).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn upload_and_analyze_read(
    State(state): State<RestApiState>,
    mut multipart: Multipart,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Upload and analyze read request");
    
    let bytes = extract_file_from_multipart(&mut multipart).await?;
    let operation = state.service.analyze_read(DocumentSource::Bytes(bytes)).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn upload_and_analyze_layout(
    State(state): State<RestApiState>,
    mut multipart: Multipart,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Upload and analyze layout request");
    
    let bytes = extract_file_from_multipart(&mut multipart).await?;
    let operation = state.service.analyze_layout(DocumentSource::Bytes(bytes)).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn upload_and_analyze_invoice(
    State(state): State<RestApiState>,
    mut multipart: Multipart,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Upload and analyze invoice request");
    
    let bytes = extract_file_from_multipart(&mut multipart).await?;
    let operation = state.service.analyze_invoice(DocumentSource::Bytes(bytes)).await?;
    
    Ok(Json(operation_to_response(operation, None)))
}

async fn get_result(
    State(state): State<RestApiState>,
    Path(operation_id): Path<String>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    info!("REST: Get result for operation: {}", operation_id);
    
    let (operation, result) = state.service.get_analysis_result(&operation_id).await
        .map_err(|e| {
            // Handle rate limiting specially
            if e.to_string().contains("429") {
                error!("Rate limit hit for operation: {}", operation_id);
                AppError::Application(e)
            } else {
                AppError::Application(e)
            }
        })?;
    
    let response = operation_to_response(operation, result);
    info!("Returning result - has data: {}", response.result.is_some());
    
    Ok(Json(response))
}

// Helper functions
fn create_domain_request(
    request: AnalyzeUrlRequest,
    model_type: ModelType,
) -> Result<AnalyzeDocumentRequest, AppError> {
    let source = DocumentSource::Url(request.document_url);
    source.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let options = AnalyzeOptions {
        locale: request.options.locale.and_then(|l| Locale::new(l).ok()),
        pages: request.options.pages.and_then(|p| PageRange::new(p).ok()),
        features: vec![],
    };
    
    Ok(AnalyzeDocumentRequest {
        source,
        model_type,
        options,
    })
}

fn operation_to_response(
    operation: AnalysisOperation,
    result: Option<AnalysisResult>,
) -> AnalyzeResponse {
    let status = format!("{:?}", operation.status).to_lowercase();
    
    // Log what we're returning
    if let Some(ref r) = result {
        info!(
            "Result has {} chars, {} pages, {} tables",
            r.content.len(),
            r.pages.len(),
            r.tables.len()
        );
    }
    
    AnalyzeResponse {
        operation_id: operation.operation_id,
        status,
        result: result.map(|r| {
            let rest_result = RestAnalysisResult {
                model_id: r.model_id.clone(),
                content: r.content.clone(),
                pages: r.pages.iter().map(|p| RestPage {
                    page_number: p.page_number,
                    width: p.width,
                    height: p.height,
                    word_count: p.words.len(),
                    line_count: p.lines.len(),
                }).collect(),
                tables: r.tables.iter().map(|t| RestTable {
                    row_count: t.row_count,
                    column_count: t.column_count,
                    cell_count: t.cells.len(),
                }).collect(),
            };
            info!("Converted to REST format - content length: {}", rest_result.content.len());
            rest_result
        }),
    }
}

async fn extract_file_from_multipart(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    let mut file_bytes = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Internal(format!("Failed to read multipart field: {}", e))
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                AppError::Internal(format!("Failed to read file data: {}", e))
            })?;
            file_bytes = data.to_vec();
            break;
        }
    }
    
    if file_bytes.is_empty() {
        return Err(AppError::Validation("No file provided".to_string()));
    }
    
    Ok(file_bytes)
}

// Error handling
#[derive(Debug)]
enum AppError {
    Validation(String),
    Internal(String),
    Application(crate::application::errors::ApplicationError),
}

impl From<crate::application::errors::ApplicationError> for AppError {
    fn from(err: crate::application::errors::ApplicationError) -> Self {
        Self::Application(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Application(err) => {
                error!("Application error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        };
        
        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

