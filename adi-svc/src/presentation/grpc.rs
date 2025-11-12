/// gRPC server implementation
/// 
/// This module implements the DocumentIntelligenceService gRPC service.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, error};
use futures::Stream;
use std::pin::Pin;

use crate::application::services::DocumentIntelligenceService;
use crate::domain::*;
use crate::generated as pb;
use crate::generated::document_intelligence_service_server::DocumentIntelligenceService as DocumentIntelligenceServiceTrait;
use super::converters::*;

/// gRPC service implementation
pub struct GrpcDocumentIntelligenceService {
    service: Arc<DocumentIntelligenceService>,
}

impl GrpcDocumentIntelligenceService {
    pub fn new(service: Arc<DocumentIntelligenceService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl DocumentIntelligenceServiceTrait for GrpcDocumentIntelligenceService {
    async fn analyze_read(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeRead request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::Read)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_layout(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeLayout request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::Layout)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_invoice(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeInvoice request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::Invoice)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_receipt(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeReceipt request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::Receipt)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_id_document(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeIdDocument request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::IdDocument)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_business_card(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeBusinessCard request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::BusinessCard)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_w2(
        &self,
        request: Request<pb::AnalyzeRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeW2 request received");
        
        let req = request.into_inner();
        let domain_request = pb_to_analyze_request(req, ModelType::W2)
            .map_err(|e| Status::invalid_argument(e))?;
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn analyze_custom(
        &self,
        request: Request<pb::AnalyzeCustomRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: AnalyzeCustom request received");
        
        let req = request.into_inner();
        let model_id = req.model_id.clone();
        
        let source = match req.source {
            Some(pb::analyze_custom_request::Source::DocumentUrl(url)) => {
                DocumentSource::Url(url)
            }
            Some(pb::analyze_custom_request::Source::DocumentBytes(bytes)) => {
                DocumentSource::Bytes(bytes)
            }
            None => return Err(Status::invalid_argument("No document source provided")),
        };
        
        let operation = self
            .service
            .analyze_custom(source, &model_id)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
    
    async fn get_analysis_result(
        &self,
        request: Request<pb::GetAnalysisResultRequest>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        let operation_id = request.into_inner().operation_id;
        info!("gRPC: GetAnalysisResult request for operation: {}", operation_id);
        
        let (operation, result) = self
            .service
            .get_analysis_result(&operation_id)
            .await
            .map_err(|e| {
                error!("Failed to get result: {}", e);
                Status::not_found(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, result);
        Ok(Response::new(response))
    }
    
    async fn upload_and_analyze(
        &self,
        request: Request<tonic::Streaming<pb::UploadRequest>>,
    ) -> Result<Response<pb::AnalyzeResponse>, Status> {
        info!("gRPC: UploadAndAnalyze request received");
        
        let mut stream = request.into_inner();
        let mut metadata: Option<pb::UploadMetadata> = None;
        let mut chunks: Vec<u8> = Vec::new();
        
        // Collect chunks
        while let Some(upload_req) = stream.message().await? {
            match upload_req.data {
                Some(pb::upload_request::Data::Metadata(meta)) => {
                    metadata = Some(meta);
                }
                Some(pb::upload_request::Data::Chunk(chunk)) => {
                    chunks.extend_from_slice(&chunk);
                }
                None => {}
            }
        }
        
        let metadata = metadata.ok_or_else(|| {
            Status::invalid_argument("No metadata provided")
        })?;
        
        let model_type = ModelType::from_string(&metadata.model_type)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        
        let domain_request = AnalyzeDocumentRequest {
            source: DocumentSource::Bytes(chunks),
            model_type,
            options: Default::default(),
        };
        
        let operation = self
            .service
            .analyze_document(domain_request)
            .await
            .map_err(|e| {
                error!("Analysis failed: {}", e);
                Status::internal(e.to_string())
            })?;
        
        let response = operation_to_pb_response(operation, None);
        Ok(Response::new(response))
    }
}

