use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub azure: AzureConfig,
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub endpoint: String,
    pub key: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub grpc_port: u16,
    pub rest_port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub upload_dir: String,
    pub max_upload_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        
        let azure = AzureConfig {
            endpoint: env::var("AZURE_DOCUMENT_INTELLIGENCE_ENDPOINT")
                .unwrap_or_else(|_| "https://your-resource.cognitiveservices.azure.com".to_string()),
            key: env::var("AZURE_DOCUMENT_INTELLIGENCE_KEY")
                .unwrap_or_else(|_| "your-api-key".to_string()),
            api_version: env::var("AZURE_API_VERSION")
                .unwrap_or_else(|_| "2024-02-29-preview".to_string()),
        };
        
        let server = ServerConfig {
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()?,
            rest_port: env::var("REST_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            host: env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
        };
        
        let storage = StorageConfig {
            upload_dir: env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            max_upload_size_mb: env::var("MAX_UPLOAD_SIZE_MB")
                .unwrap_or_else(|_| "50".to_string())
                .parse()?,
        };
        
        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://adi_user:adi_password@localhost:5432/adi_db".to_string()),
        };
        
        Ok(Self {
            azure,
            server,
            storage,
            database,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // This test will use default values if env vars are not set
        let config = Config::from_env();
        assert!(config.is_ok());
    }
}

