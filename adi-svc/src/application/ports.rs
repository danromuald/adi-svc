/// Port definitions (interfaces for adapters)
/// 
/// These traits define the contracts that adapters must implement.
/// This follows the Dependency Inversion Principle - the application
/// depends on abstractions, not concretions.

use async_trait::async_trait;
use crate::domain::{
    AnalyzeDocumentRequest, AnalysisOperation, AnalysisResult, ModelType,
};
use super::errors::ApplicationResult;

/// Port for document intelligence operations
#[async_trait]
pub trait DocumentIntelligencePort: Send + Sync {
    /// Start a document analysis operation
    async fn analyze_document(
        &self,
        request: AnalyzeDocumentRequest,
    ) -> ApplicationResult<AnalysisOperation>;
    
    /// Get the result of an analysis operation
    async fn get_analysis_result(
        &self,
        operation_id: &str,
    ) -> ApplicationResult<(AnalysisOperation, Option<AnalysisResult>)>;
    
    /// Check if a custom model exists
    async fn validate_custom_model(&self, model_id: &str) -> ApplicationResult<bool>;
}

/// Port for document storage (optional - for uploaded files)
#[async_trait]
pub trait DocumentStoragePort: Send + Sync {
    /// Store a document and return its identifier
    async fn store_document(
        &self,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> ApplicationResult<String>;
    
    /// Retrieve a document by identifier
    async fn retrieve_document(&self, document_id: &str) -> ApplicationResult<Vec<u8>>;
    
    /// Delete a document by identifier
    async fn delete_document(&self, document_id: &str) -> ApplicationResult<()>;
    
    /// Get a URL for accessing the document
    async fn get_document_url(&self, document_id: &str) -> ApplicationResult<String>;
}

/// Port for operation tracking (optional - for async operations)
#[async_trait]
pub trait OperationTrackerPort: Send + Sync {
    /// Store an operation
    async fn store_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()>;
    
    /// Retrieve an operation by ID
    async fn get_operation(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisOperation>>;
    
    /// Update an operation
    async fn update_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()>;
    
    /// Store a result for an operation
    async fn store_result(
        &self,
        operation_id: &str,
        result: &AnalysisResult,
    ) -> ApplicationResult<()>;
    
    /// Retrieve a result by operation ID
    async fn get_result(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisResult>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DocumentSource, AnalyzeOptions, OperationStatus};

    struct MockDocumentIntelligence;

    #[async_trait]
    impl DocumentIntelligencePort for MockDocumentIntelligence {
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
    async fn test_mock_port() {
        let port = MockDocumentIntelligence;
        let request = AnalyzeDocumentRequest {
            source: DocumentSource::Url("https://example.com/doc.pdf".to_string()),
            model_type: ModelType::Read,
            options: AnalyzeOptions::default(),
        };
        
        let result = port.analyze_document(request).await;
        assert!(result.is_ok());
    }
}

