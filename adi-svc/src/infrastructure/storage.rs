/// File storage adapter for document uploads
/// 
/// This adapter provides local file storage for uploaded documents.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use tracing::{debug, info};

use crate::application::errors::{ApplicationError, ApplicationResult};
use crate::application::ports::DocumentStoragePort;
use crate::infrastructure::config::StorageConfig;

/// Local file storage adapter
pub struct LocalFileStorageAdapter {
    config: StorageConfig,
}

impl LocalFileStorageAdapter {
    pub async fn new(config: StorageConfig) -> ApplicationResult<Self> {
        // Create upload directory if it doesn't exist
        fs::create_dir_all(&config.upload_dir)
            .await
            .map_err(|e| ApplicationError::Configuration(format!("Failed to create upload directory: {}", e)))?;
        
        Ok(Self { config })
    }
    
    fn get_file_path(&self, document_id: &str) -> PathBuf {
        PathBuf::from(&self.config.upload_dir).join(document_id)
    }
}

#[async_trait]
impl DocumentStoragePort for LocalFileStorageAdapter {
    async fn store_document(
        &self,
        filename: &str,
        _content_type: &str,
        data: Vec<u8>,
    ) -> ApplicationResult<String> {
        // Check size limit
        let max_bytes = self.config.max_upload_size_mb * 1024 * 1024;
        if data.len() > max_bytes {
            return Err(ApplicationError::Internal(format!(
                "File too large: {} bytes (max: {} bytes)",
                data.len(),
                max_bytes
            )));
        }
        
        // Generate unique ID
        let document_id = format!("{}_{}", Uuid::new_v4(), filename);
        let file_path = self.get_file_path(&document_id);
        
        debug!("Storing document: {} ({} bytes)", document_id, data.len());
        
        // Write file
        fs::write(&file_path, data)
            .await
            .map_err(|e| ApplicationError::Internal(format!("Failed to write file: {}", e)))?;
        
        info!("Document stored successfully: {}", document_id);
        Ok(document_id)
    }
    
    async fn retrieve_document(&self, document_id: &str) -> ApplicationResult<Vec<u8>> {
        let file_path = self.get_file_path(document_id);
        
        debug!("Retrieving document: {}", document_id);
        
        fs::read(&file_path)
            .await
            .map_err(|e| ApplicationError::Internal(format!("Failed to read file: {}", e)))
    }
    
    async fn delete_document(&self, document_id: &str) -> ApplicationResult<()> {
        let file_path = self.get_file_path(document_id);
        
        debug!("Deleting document: {}", document_id);
        
        fs::remove_file(&file_path)
            .await
            .map_err(|e| ApplicationError::Internal(format!("Failed to delete file: {}", e)))?;
        
        info!("Document deleted successfully: {}", document_id);
        Ok(())
    }
    
    async fn get_document_url(&self, document_id: &str) -> ApplicationResult<String> {
        // For local storage, we return a file:// URL
        // In production, this would be an HTTP URL to a file server
        let file_path = self.get_file_path(document_id);
        Ok(format!("file://{}", file_path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            upload_dir: temp_dir.path().to_str().unwrap().to_string(),
            max_upload_size_mb: 10,
        };
        
        let storage = LocalFileStorageAdapter::new(config).await.unwrap();
        
        let data = b"test data".to_vec();
        let doc_id = storage
            .store_document("test.txt", "text/plain", data.clone())
            .await
            .unwrap();
        
        let retrieved = storage.retrieve_document(&doc_id).await.unwrap();
        assert_eq!(retrieved, data);
        
        storage.delete_document(&doc_id).await.unwrap();
    }
}

