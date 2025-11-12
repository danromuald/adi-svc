/// PostgreSQL-based operation tracker
/// 
/// This adapter stores operations and results in PostgreSQL for persistence
/// across service restarts and multi-instance deployments.

use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use tracing::{debug, info, error};

use crate::application::errors::{ApplicationError, ApplicationResult};
use crate::application::ports::OperationTrackerPort;
use crate::domain::{AnalysisOperation, AnalysisResult, OperationStatus};

/// PostgreSQL operation tracker
pub struct PostgresOperationTracker {
    pool: PgPool,
}

impl PostgresOperationTracker {
    pub async fn new(database_url: &str) -> ApplicationResult<Self> {
        info!("Connecting to PostgreSQL database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| ApplicationError::Configuration(format!("Database connection failed: {}", e)))?;
        
        info!("✓ Connected to PostgreSQL database");
        
        Ok(Self { pool })
    }
}

#[async_trait]
impl OperationTrackerPort for PostgresOperationTracker {
    async fn store_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()> {
        debug!("Storing operation: {}", operation.operation_id);
        
        let status_str = format!("{:?}", operation.status).to_lowercase();
        let model_type_str = format!("{:?}", operation.model_type);
        
        sqlx::query(
            r#"
            INSERT INTO operations (operation_id, status, model_type, created_at, last_updated)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (operation_id) DO UPDATE
            SET status = $2, last_updated = $5
            "#
        )
        .bind(&operation.operation_id)
        .bind(&status_str)
        .bind(&model_type_str)
        .bind(operation.created_at)
        .bind(operation.last_updated)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(format!("Failed to store operation: {}", e)))?;
        
        info!("Operation stored: {}", operation.operation_id);
        Ok(())
    }
    
    async fn get_operation(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisOperation>> {
        debug!("Getting operation: {}", operation_id);
        
        let row = sqlx::query(
            r#"
            SELECT operation_id, status, model_type, created_at, last_updated
            FROM operations
            WHERE operation_id = $1
            "#
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(format!("Failed to get operation: {}", e)))?;
        
        if let Some(row) = row {
            let operation_id: String = row.get(0);
            let status_str: String = row.get(1);
            let model_type_str: String = row.get(2);
            let created_at: chrono::DateTime<chrono::Utc> = row.get(3);
            let last_updated: chrono::DateTime<chrono::Utc> = row.get(4);
            
            let status = match status_str.as_str() {
                "notstarted" => OperationStatus::NotStarted,
                "running" => OperationStatus::Running,
                "succeeded" => OperationStatus::Succeeded,
                "failed" => OperationStatus::Failed,
                "canceled" => OperationStatus::Canceled,
                _ => OperationStatus::NotStarted,
            };
            
            let model_type = crate::domain::ModelType::from_string(&model_type_str)
                .unwrap_or(crate::domain::ModelType::Read);
            
            Ok(Some(AnalysisOperation {
                operation_id,
                status,
                created_at,
                last_updated,
                model_type,
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn update_operation(&self, operation: &AnalysisOperation) -> ApplicationResult<()> {
        debug!("Updating operation: {}", operation.operation_id);
        
        let status_str = format!("{:?}", operation.status).to_lowercase();
        
        sqlx::query(
            r#"
            UPDATE operations
            SET status = $1, last_updated = $2
            WHERE operation_id = $3
            "#
        )
        .bind(&status_str)
        .bind(operation.last_updated)
        .bind(&operation.operation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(format!("Failed to update operation: {}", e)))?;
        
        info!("Operation updated: {}", operation.operation_id);
        Ok(())
    }
    
    async fn store_result(
        &self,
        operation_id: &str,
        result: &AnalysisResult,
    ) -> ApplicationResult<()> {
        debug!("Storing result for operation: {}", operation_id);
        
        // Serialize complex data as JSON
        let pages_json = serde_json::to_value(&result.pages)
            .map_err(|e| ApplicationError::Internal(format!("Failed to serialize pages: {}", e)))?;
        let tables_json = serde_json::to_value(&result.tables)
            .map_err(|e| ApplicationError::Internal(format!("Failed to serialize tables: {}", e)))?;
        let kvp_json = serde_json::to_value(&result.key_value_pairs)
            .map_err(|e| ApplicationError::Internal(format!("Failed to serialize key-value pairs: {}", e)))?;
        let docs_json = serde_json::to_value(&result.documents)
            .map_err(|e| ApplicationError::Internal(format!("Failed to serialize documents: {}", e)))?;
        
        sqlx::query(
            r#"
            INSERT INTO results (
                operation_id, model_id, api_version, content,
                pages_data, tables_data, key_value_pairs_data, documents_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (operation_id) DO UPDATE
            SET model_id = $2, api_version = $3, content = $4,
                pages_data = $5, tables_data = $6, key_value_pairs_data = $7, documents_data = $8
            "#
        )
        .bind(operation_id)
        .bind(&result.model_id)
        .bind(&result.api_version)
        .bind(&result.content)
        .bind(pages_json)
        .bind(tables_json)
        .bind(kvp_json)
        .bind(docs_json)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(format!("Failed to store result: {}", e)))?;
        
        info!("Result stored for operation: {}", operation_id);
        Ok(())
    }
    
    async fn get_result(&self, operation_id: &str) -> ApplicationResult<Option<AnalysisResult>> {
        debug!("Getting result for operation: {}", operation_id);
        
        let row = sqlx::query(
            r#"
            SELECT model_id, api_version, content, pages_data, tables_data, key_value_pairs_data, documents_data
            FROM results
            WHERE operation_id = $1
            "#
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(format!("Failed to get result: {}", e)))?;
        
        if let Some(row) = row {
            let model_id: String = row.get(0);
            let api_version: String = row.get(1);
            let content: String = row.get(2);
            let pages_json: serde_json::Value = row.get(3);
            let tables_json: serde_json::Value = row.get(4);
            let kvp_json: serde_json::Value = row.get(5);
            let docs_json: serde_json::Value = row.get(6);
            
            let pages = serde_json::from_value(pages_json)
                .unwrap_or_default();
            let tables = serde_json::from_value(tables_json)
                .unwrap_or_default();
            let key_value_pairs = serde_json::from_value(kvp_json)
                .unwrap_or_default();
            let documents = serde_json::from_value(docs_json)
                .unwrap_or_default();
            
            Ok(Some(AnalysisResult {
                model_id,
                api_version,
                content,
                pages,
                tables,
                key_value_pairs,
                documents,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests require a running PostgreSQL instance
    // Run with: docker run -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15-alpine
    
    #[tokio::test]
    #[ignore] // Only run with --ignored flag when database is available
    async fn test_store_and_get_operation() {
        let tracker = PostgresOperationTracker::new(
            "postgresql://postgres:password@localhost:5432/postgres"
        ).await.unwrap();
        
        let operation = AnalysisOperation::new(crate::domain::ModelType::Read);
        
        tracker.store_operation(&operation).await.unwrap();
        let retrieved = tracker.get_operation(&operation.operation_id).await.unwrap();
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().operation_id, operation.operation_id);
    }
}

