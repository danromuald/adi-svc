use super::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Model type for document analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Read,
    Layout,
    Invoice,
    Receipt,
    IdDocument,
    BusinessCard,
    W2,
    Custom,
}

impl ModelType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Read => "prebuilt-read",
            Self::Layout => "prebuilt-layout",
            Self::Invoice => "prebuilt-invoice",
            Self::Receipt => "prebuilt-receipt",
            Self::IdDocument => "prebuilt-idDocument",
            Self::BusinessCard => "prebuilt-businessCard",
            Self::W2 => "prebuilt-tax.us.w2",
            Self::Custom => "custom",
        }
    }
    
    pub fn from_string(s: &str) -> DomainResult<Self> {
        match s.to_lowercase().as_str() {
            "read" | "prebuilt-read" => Ok(Self::Read),
            "layout" | "prebuilt-layout" => Ok(Self::Layout),
            "invoice" | "prebuilt-invoice" => Ok(Self::Invoice),
            "receipt" | "prebuilt-receipt" => Ok(Self::Receipt),
            "iddocument" | "prebuilt-iddocument" => Ok(Self::IdDocument),
            "businesscard" | "prebuilt-businesscard" => Ok(Self::BusinessCard),
            "w2" | "prebuilt-tax.us.w2" => Ok(Self::W2),
            "custom" => Ok(Self::Custom),
            _ => Err(DomainError::InvalidModelType(s.to_string())),
        }
    }
}

impl FromStr for ModelType {
    type Err = DomainError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Document source - either URL or bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentSource {
    Url(String),
    Bytes(Vec<u8>),
}

impl DocumentSource {
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            Self::Url(url) => {
                if url.is_empty() {
                    return Err(DomainError::ValidationError("URL cannot be empty".to_string()));
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(DomainError::ValidationError(
                        "URL must start with http:// or https://".to_string(),
                    ));
                }
                Ok(())
            }
            Self::Bytes(bytes) => {
                if bytes.is_empty() {
                    return Err(DomainError::ValidationError("Document bytes cannot be empty".to_string()));
                }
                const MAX_SIZE: usize = 500 * 1024 * 1024; // 500MB
                if bytes.len() > MAX_SIZE {
                    return Err(DomainError::DocumentTooLarge {
                        size: bytes.len(),
                        max: MAX_SIZE,
                    });
                }
                Ok(())
            }
        }
    }
}

/// Locale for document analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locale(String);

impl Locale {
    pub fn new(locale: impl Into<String>) -> DomainResult<Self> {
        let locale = locale.into();
        // Basic validation - locale should be in format like "en-US"
        if locale.len() >= 2 && !locale.is_empty() {
            Ok(Self(locale))
        } else {
            Err(DomainError::InvalidLocale(locale))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self("en-US".to_string())
    }
}

/// Page range for document analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRange(Vec<String>);

impl PageRange {
    pub fn new(pages: Vec<String>) -> DomainResult<Self> {
        // Validate page ranges (e.g., "1", "1-3", "1,3,5-7")
        for page in &pages {
            if page.is_empty() {
                return Err(DomainError::InvalidPageRange("Empty page range".to_string()));
            }
        }
        Ok(Self(pages))
    }
    
    pub fn all() -> Self {
        Self(vec![])
    }
    
    pub fn as_vec(&self) -> &[String] {
        &self.0
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for PageRange {
    fn default() -> Self {
        Self::all()
    }
}

/// Additional features that can be enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisFeature {
    OcrHighResolution,
    Languages,
    Barcodes,
    Formulas,
    StyleFont,
    KeyValuePairs,
}

impl AnalysisFeature {
    pub fn as_str(&self) -> &str {
        match self {
            Self::OcrHighResolution => "ocrHighResolution",
            Self::Languages => "languages",
            Self::Barcodes => "barcodes",
            Self::Formulas => "formulas",
            Self::StyleFont => "styleFont",
            Self::KeyValuePairs => "keyValuePairs",
        }
    }
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    NotStarted,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

impl OperationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_conversion() {
        assert_eq!(ModelType::Read.as_str(), "prebuilt-read");
        assert_eq!(ModelType::from_string("read").unwrap(), ModelType::Read);
        assert_eq!(ModelType::from_string("invoice").unwrap(), ModelType::Invoice);
    }

    #[test]
    fn test_document_source_validation() {
        let valid_url = DocumentSource::Url("https://example.com/doc.pdf".to_string());
        assert!(valid_url.validate().is_ok());

        let invalid_url = DocumentSource::Url("".to_string());
        assert!(invalid_url.validate().is_err());

        let valid_bytes = DocumentSource::Bytes(vec![1, 2, 3]);
        assert!(valid_bytes.validate().is_ok());

        let empty_bytes = DocumentSource::Bytes(vec![]);
        assert!(empty_bytes.validate().is_err());
    }

    #[test]
    fn test_locale() {
        let locale = Locale::new("en-US").unwrap();
        assert_eq!(locale.as_str(), "en-US");

        assert!(Locale::new("").is_err());
    }

    #[test]
    fn test_operation_status() {
        assert!(!OperationStatus::Running.is_terminal());
        assert!(OperationStatus::Succeeded.is_terminal());
        assert!(OperationStatus::Failed.is_terminal());
    }
}

