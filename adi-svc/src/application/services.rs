// Copyright 2025 Dan Mbanga
// Licensed under the Apache License, Version 2.0

/// Application services - Use case implementations
/// 
/// These services orchestrate domain objects and ports to implement
/// the application's use cases.

use std::sync::Arc;
use crate::domain::{
    AnalyzeDocumentRequest, AnalysisOperation, AnalysisResult, DocumentSource, ModelType,
};
use super::errors::{ApplicationError, ApplicationResult};
use super::ports::{DocumentIntelligencePort, DocumentStoragePort, OperationTrackerPort};
use tracing::{info, warn, error};

/// Main document intelligence service
pub struct DocumentIntelligenceService {
    intelligence_adapter: Arc<dyn DocumentIntelligencePort>,
    storage_adapter: Option<Arc<dyn DocumentStoragePort>>,
    tracker_adapter: Option<Arc<dyn OperationTrackerPort>>,
}

impl DocumentIntelligenceService {
    pub fn new(
        intelligence_adapter: Arc<dyn DocumentIntelligencePort>,
        storage_adapter: Option<Arc<dyn DocumentStoragePort>>,
        tracker_adapter: Option<Arc<dyn OperationTrackerPort>>,
    ) -> Self {
        Self {
            intelligence_adapter,
            storage_adapter,
            tracker_adapter,
        }
    }
    
    /// Analyze a document using the specified model
    pub async fn analyze_document(
        &self,
        mut request: AnalyzeDocumentRequest,
    ) -> ApplicationResult<AnalysisOperation> {
        info!("Starting document analysis with model: {:?}", request.model_type);
        
        // Validate the request
        request.source.validate().map_err(ApplicationError::Domain)?;
        
        // If document is provided as bytes and storage is available, store it for record-keeping
        // but keep the bytes for Azure API call
        if let DocumentSource::Bytes(ref bytes) = request.source {
            if let Some(storage) = &self.storage_adapter {
                info!("Storing document bytes for record-keeping");
                let _doc_id = storage
                    .store_document(
                        "uploaded_document",
                        "application/octet-stream",
                        bytes.clone(),
                    )
                    .await?;
                // Note: We keep request.source as Bytes - don't convert to file:// URL
                // Azure needs the base64-encoded bytes, not a local file path
            }
        }
        
        // Start analysis
        let operation = self.intelligence_adapter.analyze_document(request).await?;
        
        // Track operation if tracker is available
        if let Some(tracker) = &self.tracker_adapter {
            tracker.store_operation(&operation).await?;
        }
        
        info!("Document analysis started: operation_id={}", operation.operation_id);
        Ok(operation)
    }
    
    /// Get the result of an analysis operation
    pub async fn get_analysis_result(
        &self,
        operation_id: &str,
    ) -> ApplicationResult<(AnalysisOperation, Option<AnalysisResult>)> {
        info!("Retrieving analysis result: operation_id={}", operation_id);
        
        // ALWAYS check tracker first
        let stored_operation = if let Some(tracker) = &self.tracker_adapter {
            tracker.get_operation(operation_id).await?
        } else {
            None
        };
        
        // If we have a stored operation with terminal status and result, return from cache
        if let Some(ref op) = stored_operation {
            if op.status.is_terminal() {
                if let Some(tracker) = &self.tracker_adapter {
                    if let Some(result) = tracker.get_result(operation_id).await? {
                        info!("Returning cached result for operation: {}", operation_id);
                        return Ok((op.clone(), Some(result)));
                    }
                }
            }
        }
        
        // Query Azure
        let (mut operation, result) = self
            .intelligence_adapter
            .get_analysis_result(operation_id)
            .await?;
        
        // Update tracker if available
        if let Some(tracker) = &self.tracker_adapter {
            tracker.update_operation(&operation).await?;
            if let Some(ref result) = result {
                tracker.store_result(operation_id, result).await?;
            }
        }
        
        // Use stored model_type if available
        if let Some(stored_op) = stored_operation {
            operation.model_type = stored_op.model_type;
        }
        
        Ok((operation, result))
    }
    
    /// Analyze with Read model
    pub async fn analyze_read(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::Read,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze with Layout model
    pub async fn analyze_layout(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::Layout,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze invoice
    pub async fn analyze_invoice(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::Invoice,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze receipt
    pub async fn analyze_receipt(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::Receipt,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze ID document
    pub async fn analyze_id_document(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::IdDocument,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze business card
    pub async fn analyze_business_card(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::BusinessCard,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze W-2 tax form
    pub async fn analyze_w2(
        &self,
        source: DocumentSource,
    ) -> ApplicationResult<AnalysisOperation> {
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::W2,
            options: Default::default(),
        };
        self.analyze_document(request).await
    }
    
    /// Analyze with custom model
    pub async fn analyze_custom(
        &self,
        source: DocumentSource,
        model_id: &str,
    ) -> ApplicationResult<AnalysisOperation> {
        info!("Validating custom model: {}", model_id);
        
        // Validate custom model exists
        let exists = self
            .intelligence_adapter
            .validate_custom_model(model_id)
            .await?;
        
        if !exists {
            return Err(ApplicationError::AnalysisFailed(
                format!("Custom model not found: {}", model_id),
            ));
        }
        
        let request = AnalyzeDocumentRequest {
            source,
            model_type: ModelType::Custom,
            options: Default::default(),
        };
        
        self.analyze_document(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::DocumentIntelligencePort;
    use crate::domain::{OperationStatus, AnalyzeOptions};
    use async_trait::async_trait;

    struct MockIntelligenceAdapter;

    #[async_trait]
    impl DocumentIntelligencePort for MockIntelligenceAdapter {
        async fn analyze_document(
            &self,
            _request: AnalyzeDocumentRequest,
        ) -> ApplicationResult<AnalysisOperation> {
            Ok(AnalysisOperation::new(ModelType::Read))
        }

        async fn get_analysis_result(
            &self,
            _operation_id: &str,
        ) -> ApplicationResult<(AnalysisOperation, Option<AnalysisResult>)> {
            let mut op = AnalysisOperation::new(ModelType::Read);
            op.update_status(OperationStatus::Succeeded);
            Ok((op, Some(AnalysisResult::default())))
        }

        async fn validate_custom_model(&self, _model_id: &str) -> ApplicationResult<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_analyze_read() {
        let adapter = Arc::new(MockIntelligenceAdapter);
        let service = DocumentIntelligenceService::new(adapter, None, None);
        
        let result = service
            .analyze_read(DocumentSource::Url("https://example.com/doc.pdf".to_string()))
            .await;
        
        assert!(result.is_ok());
        let operation = result.unwrap();
        assert_eq!(operation.model_type, ModelType::Read);
    }
}

