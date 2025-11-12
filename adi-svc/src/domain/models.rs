use super::value_objects::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request to analyze a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeDocumentRequest {
    pub source: DocumentSource,
    pub model_type: ModelType,
    pub options: AnalyzeOptions,
}

/// Options for document analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    pub locale: Option<Locale>,
    pub pages: Option<PageRange>,
    pub features: Vec<AnalysisFeature>,
}

/// Analysis operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOperation {
    pub operation_id: String,
    pub status: OperationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub model_type: ModelType,
}

impl AnalysisOperation {
    pub fn new(model_type: ModelType) -> Self {
        let now = chrono::Utc::now();
        Self {
            operation_id: Uuid::new_v4().to_string(),
            status: OperationStatus::NotStarted,
            created_at: now,
            last_updated: now,
            model_type,
        }
    }
    
    pub fn update_status(&mut self, status: OperationStatus) {
        self.status = status;
        self.last_updated = chrono::Utc::now();
    }
}

/// Complete analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub model_id: String,
    pub api_version: String,
    pub content: String,
    pub pages: Vec<DocumentPage>,
    pub tables: Vec<DocumentTable>,
    pub key_value_pairs: Vec<KeyValuePair>,
    pub documents: Vec<ExtractedDocument>,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            api_version: "2024-02-29-preview".to_string(),
            content: String::new(),
            pages: Vec::new(),
            tables: Vec::new(),
            key_value_pairs: Vec::new(),
            documents: Vec::new(),
        }
    }
}

/// Document page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPage {
    pub page_number: i32,
    pub angle: f32,
    pub width: f32,
    pub height: f32,
    pub unit: String,
    pub words: Vec<DocumentWord>,
    pub lines: Vec<DocumentLine>,
    pub selection_marks: Vec<SelectionMark>,
}

/// Word in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWord {
    pub content: String,
    pub polygon: Vec<Point>,
    pub confidence: f32,
    pub span: Span,
}

/// Line in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLine {
    pub content: String,
    pub polygon: Vec<Point>,
    pub spans: Vec<Span>,
}

/// Selection mark (checkbox, radio button)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionMark {
    pub state: SelectionMarkState,
    pub polygon: Vec<Point>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionMarkState {
    Selected,
    Unselected,
}

/// Point in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Span (reference to content)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Span {
    pub offset: i32,
    pub length: i32,
}

/// Table in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    pub row_count: i32,
    pub column_count: i32,
    pub cells: Vec<TableCell>,
}

/// Table cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub kind: CellKind,
    pub row_index: i32,
    pub column_index: i32,
    pub row_span: i32,
    pub column_span: i32,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CellKind {
    Content,
    RowHeader,
    ColumnHeader,
    StubHead,
    Description,
}

/// Key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub confidence: f32,
}

/// Extracted document (for prebuilt models)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDocument {
    pub doc_type: String,
    pub fields: HashMap<String, DocumentField>,
    pub confidence: f32,
}

/// Document field with typed value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DocumentField {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "number")]
    Number(f64),
    #[serde(rename = "integer")]
    Integer(i32),
    #[serde(rename = "date")]
    Date(chrono::NaiveDate),
    #[serde(rename = "time")]
    Time(chrono::NaiveTime),
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "array")]
    Array(Vec<DocumentField>),
    #[serde(rename = "object")]
    Object(HashMap<String, DocumentField>),
}

impl DocumentField {
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_operation_creation() {
        let op = AnalysisOperation::new(ModelType::Read);
        assert_eq!(op.status, OperationStatus::NotStarted);
        assert_eq!(op.model_type, ModelType::Read);
        assert!(!op.operation_id.is_empty());
    }

    #[test]
    fn test_analysis_operation_status_update() {
        let mut op = AnalysisOperation::new(ModelType::Layout);
        let initial_time = op.last_updated;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        op.update_status(OperationStatus::Running);
        
        assert_eq!(op.status, OperationStatus::Running);
        assert!(op.last_updated > initial_time);
    }

    #[test]
    fn test_document_field_accessors() {
        let string_field = DocumentField::String("test".to_string());
        assert_eq!(string_field.as_string(), Some("test"));
        assert_eq!(string_field.as_number(), None);

        let number_field = DocumentField::Number(42.5);
        assert_eq!(number_field.as_number(), Some(42.5));
        assert_eq!(number_field.as_string(), None);
    }
}

