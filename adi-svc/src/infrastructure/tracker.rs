/// Operation tracker adapter
/// 
/// This adapter provides in-memory operation tracking.
/// In production, this would use a database like Redis or PostgreSQL.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::application::errors::{ApplicationError, ApplicationResult};
use crate::application::ports::OperationTrackerPort;
use crate::domain::{AnalysisOperation, AnalysisResult};

/// In-memory operation tracker
pub struct InMemoryOperationTracker {
    operations: Arc<RwLock<HashMap<String, AnalysisOperation>>>,
    results: Arc<RwLock<HashMap<String, AnalysisResult>>>,
}

impl InMemoryOperationTracker {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryOperationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OperationTrackerPort for InMemoryOperationTracker {
    async fn store_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()> {
        debug!("Storing operation: {}", operation.operation_id);
        let mut operations = self.operations.write().await;
        operations.insert(operation.operation_id.clone(), operation.clone());
        info!("Operation stored: {}", operation.operation_id);
        Ok(())
    }
    
    async fn get_operation(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisOperation>> {
        debug!("Getting operation: {}", operation_id);
        let operations = self.operations.read().await;
        Ok(operations.get(operation_id).cloned())
    }
    
    async fn update_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()> {
        debug!("Updating operation: {}", operation.operation_id);
        let mut operations = self.operations.write().await;
        operations.insert(operation.operation_id.clone(), operation.clone());
        info!("Operation updated: {}", operation.operation_id);
        Ok(())
    }
    
    async fn store_result(
        &self,
        operation_id: &str,
        result: &AnalysisResult,
    ) -> ApplicationResult<()> {
        debug!("Storing result for operation: {}", operation_id);
        let mut results = self.results.write().await;
        results.insert(operation_id.to_string(), result.clone());
        info!("Result stored for operation: {}", operation_id);
        Ok(())
    }
    
    async fn get_result(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisResult>> {
        debug!("Getting result for operation: {}", operation_id);
        let results = self.results.read().await;
        Ok(results.get(operation_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModelType, OperationStatus};

    #[tokio::test]
    async fn test_store_and_get_operation() {
        let tracker = InMemoryOperationTracker::new();
        let operation = AnalysisOperation::new(ModelType::Read);
        
        tracker.store_operation(&operation).await.unwrap();
        let retrieved = tracker
            .get_operation(&operation.operation_id)
            .await
            .unwrap();
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().operation_id, operation.operation_id);
    }

    #[tokio::test]
    async fn test_update_operation() {
        let tracker = InMemoryOperationTracker::new();
        let mut operation = AnalysisOperation::new(ModelType::Layout);
        
        tracker.store_operation(&operation).await.unwrap();
        operation.update_status(OperationStatus::Succeeded);
        tracker.update_operation(&operation).await.unwrap();
        
        let retrieved = tracker
            .get_operation(&operation.operation_id)
            .await
            .unwrap()
            .unwrap();
        
        assert_eq!(retrieved.status, OperationStatus::Succeeded);
    }

    #[tokio::test]
    async fn test_store_and_get_result() {
        let tracker = InMemoryOperationTracker::new();
        let operation_id = "test-op-123";
        let result = AnalysisResult::default();
        
        tracker.store_result(operation_id, &result).await.unwrap();
        let retrieved = tracker.get_result(operation_id).await.unwrap();
        
        assert!(retrieved.is_some());
    }
}

