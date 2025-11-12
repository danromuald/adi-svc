// Copyright 2025 Dan Mbanga
// Licensed under the Apache License, Version 2.0

/// Azure AI Document Intelligence adapter
/// 
/// This adapter implements the DocumentIntelligencePort using
/// the Azure REST API.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn, error};
use base64::{Engine as _, engine::general_purpose};

use crate::application::errors::{ApplicationError, ApplicationResult};
use crate::application::ports::DocumentIntelligencePort;
use crate::domain::*;
use crate::infrastructure::config::AzureConfig;

/// Azure Document Intelligence adapter
pub struct AzureDocumentIntelligenceAdapter {
    config: AzureConfig,
    client: Client,
}

impl AzureDocumentIntelligenceAdapter {
    pub fn new(config: AzureConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    fn build_url(&self, path: &str) -> String {
        format!(
            "{}/documentintelligence/documentModels/{}:analyze?api-version={}",
            self.config.endpoint.trim_end_matches('/'),
            path,
            self.config.api_version
        )
    }
    
    fn build_result_url(&self, model_id: &str, result_id: &str) -> String {
        format!(
            "{}/documentintelligence/documentModels/{}/analyzeResults/{}?api-version={}",
            self.config.endpoint.trim_end_matches('/'),
            model_id,
            result_id,
            self.config.api_version
        )
    }
    
    async fn submit_analysis(
        &self,
        model_id: &str,
        request: &AnalyzeDocumentRequest,
    ) -> ApplicationResult<String> {
        let url = self.build_url(model_id);
        debug!("Submitting analysis to: {}", url);
        
        let body = match &request.source {
            DocumentSource::Url(doc_url) => {
                AzureAnalyzeRequest::Url { url_source: doc_url.clone() }
            }
            DocumentSource::Bytes(bytes) => {
                AzureAnalyzeRequest::Base64 {
                    base64_source: general_purpose::STANDARD.encode(bytes),
                }
            }
        };
        
        let response = self
            .client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", &self.config.key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ApplicationError::AzureService(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Azure API error: {} - {}", status, error_text);
            return Err(ApplicationError::AzureService(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }
        
        // Extract operation location from headers
        let operation_location = response
            .headers()
            .get("operation-location")
            .or_else(|| response.headers().get("apim-request-id"))
            .ok_or_else(|| {
                ApplicationError::AzureService("No operation location in response".to_string())
            })?
            .to_str()
            .map_err(|e| ApplicationError::AzureService(format!("Invalid header value: {}", e)))?
            .to_string();
        
        // Extract operation ID from the location URL (remove query parameters)
        let operation_id = operation_location
            .split('/')
            .last()
            .unwrap_or(&operation_location)
            .split('?')
            .next()
            .unwrap_or(&operation_location)
            .to_string();
        
        info!("Analysis submitted successfully: operation_id={}", operation_id);
        Ok(operation_id)
    }
    
    async fn poll_result(
        &self,
        model_id: &str,
        operation_id: &str,
    ) -> ApplicationResult<AzureAnalyzeResult> {
        let url = self.build_result_url(model_id, operation_id);
        debug!("Polling result from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Ocp-Apim-Subscription-Key", &self.config.key)
            .send()
            .await
            .map_err(|e| ApplicationError::AzureService(format!("Request failed: {}", e)))?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(ApplicationError::OperationNotFound(operation_id.to_string()));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApplicationError::AzureService(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }
        
        let result: AzureAnalyzeResult = response
            .json()
            .await
            .map_err(|e| ApplicationError::AzureService(format!("Failed to parse response: {}", e)))?;
        
        Ok(result)
    }
    
    fn convert_azure_result(&self, azure_result: AzureAnalyzeResult) -> AnalysisResult {
        AnalysisResult {
            model_id: azure_result.model_id.unwrap_or_default(),
            api_version: self.config.api_version.clone(),
            content: azure_result.content.unwrap_or_default(),
            pages: azure_result
                .pages
                .unwrap_or_default()
                .into_iter()
                .map(Self::convert_page)
                .collect(),
            tables: azure_result
                .tables
                .unwrap_or_default()
                .into_iter()
                .map(Self::convert_table)
                .collect(),
            key_value_pairs: azure_result
                .key_value_pairs
                .unwrap_or_default()
                .into_iter()
                .map(Self::convert_kvp)
                .collect(),
            documents: azure_result
                .documents
                .unwrap_or_default()
                .into_iter()
                .map(Self::convert_document)
                .collect(),
        }
    }
    
    fn convert_page(page: AzurePage) -> DocumentPage {
        DocumentPage {
            page_number: page.page_number,
            angle: page.angle.unwrap_or(0.0),
            width: page.width,
            height: page.height,
            unit: page.unit,
            words: page.words.unwrap_or_default().into_iter().map(Self::convert_word).collect(),
            lines: page.lines.unwrap_or_default().into_iter().map(Self::convert_line).collect(),
            selection_marks: page.selection_marks.unwrap_or_default().into_iter().map(Self::convert_selection_mark).collect(),
        }
    }
    
    fn convert_word(word: AzureWord) -> DocumentWord {
        DocumentWord {
            content: word.content,
            polygon: word.polygon.into_iter().map(|p| Point { x: p[0], y: p[1] }).collect(),
            confidence: word.confidence.unwrap_or(1.0),
            span: Span {
                offset: word.span.offset,
                length: word.span.length,
            },
        }
    }
    
    fn convert_line(line: AzureLine) -> DocumentLine {
        DocumentLine {
            content: line.content,
            polygon: line.polygon.into_iter().map(|p| Point { x: p[0], y: p[1] }).collect(),
            spans: line.spans.into_iter().map(|s| Span {
                offset: s.offset,
                length: s.length,
            }).collect(),
        }
    }
    
    fn convert_selection_mark(mark: AzureSelectionMark) -> SelectionMark {
        SelectionMark {
            state: match mark.state.as_str() {
                "selected" => SelectionMarkState::Selected,
                _ => SelectionMarkState::Unselected,
            },
            polygon: mark.polygon.into_iter().map(|p| Point { x: p[0], y: p[1] }).collect(),
            confidence: mark.confidence.unwrap_or(1.0),
        }
    }
    
    fn convert_table(table: AzureTable) -> DocumentTable {
        DocumentTable {
            row_count: table.row_count,
            column_count: table.column_count,
            cells: table.cells.into_iter().map(Self::convert_cell).collect(),
        }
    }
    
    fn convert_cell(cell: AzureTableCell) -> TableCell {
        TableCell {
            kind: match cell.kind.as_deref() {
                Some("rowHeader") => CellKind::RowHeader,
                Some("columnHeader") => CellKind::ColumnHeader,
                _ => CellKind::Content,
            },
            row_index: cell.row_index,
            column_index: cell.column_index,
            row_span: cell.row_span.unwrap_or(1),
            column_span: cell.column_span.unwrap_or(1),
            content: cell.content,
        }
    }
    
    fn convert_kvp(kvp: AzureKeyValuePair) -> KeyValuePair {
        KeyValuePair {
            key: kvp.key.content.unwrap_or_default(),
            value: kvp.value.content.unwrap_or_default(),
            confidence: kvp.confidence.unwrap_or(1.0),
        }
    }
    
    fn convert_document(doc: AzureDocument) -> ExtractedDocument {
        ExtractedDocument {
            doc_type: doc.doc_type,
            fields: doc
                .fields
                .into_iter()
                .filter_map(|(k, v)| v.content.map(|c| (k, DocumentField::String(c))))
                .collect(),
            confidence: doc.confidence.unwrap_or(1.0),
        }
    }
}

#[async_trait]
impl DocumentIntelligencePort for AzureDocumentIntelligenceAdapter {
    async fn analyze_document(
        &self,
        request: AnalyzeDocumentRequest,
    ) -> ApplicationResult<AnalysisOperation> {
        let model_id = request.model_type.as_str();
        
        info!("Starting analysis with model: {}", model_id);
        let operation_id = self.submit_analysis(model_id, &request).await?;
        
        let mut operation = AnalysisOperation::new(request.model_type);
        operation.operation_id = operation_id.clone();
        operation.update_status(OperationStatus::Running);
        
        // Store the model ID with the operation for later retrieval
        // We'll use a simple format: operation_id contains just the UUID part
        
        Ok(operation)
    }
    
    async fn get_analysis_result(
        &self,
        operation_id: &str,
    ) -> ApplicationResult<(AnalysisOperation, Option<AnalysisResult>)> {
        // This is a fallback - should prefer using model type from database
        // Try most common model only
        warn!("get_analysis_result called without model context");
        
        let model_id = "prebuilt-read";
        let model_type = ModelType::Read;
        
        match self.poll_result(model_id, operation_id).await {
            Ok(azure_result) => {
                let mut operation = AnalysisOperation::new(model_type);
                operation.operation_id = operation_id.to_string();
                
                let status = match azure_result.status.as_str() {
                    "succeeded" => OperationStatus::Succeeded,
                    "failed" => OperationStatus::Failed,
                    "running" => OperationStatus::Running,
                    _ => OperationStatus::Running,
                };
                
                operation.update_status(status);
                
                let result = if status == OperationStatus::Succeeded {
                    Some(self.convert_azure_result(azure_result))
                } else {
                    None
                };
                
                Ok((operation, result))
            }
            Err(e) => Err(e),
        }
    }
    
    
    async fn validate_custom_model(&self, model_id: &str) -> ApplicationResult<bool> {
        // In a full implementation, this would check if the model exists
        // For now, we'll assume custom models exist
        info!("Validating custom model: {}", model_id);
        Ok(true)
    }
}

// Azure API DTOs
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AzureAnalyzeRequest {
    Url { #[serde(rename = "urlSource")] url_source: String },
    Base64 { #[serde(rename = "base64Source")] base64_source: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureAnalyzeResult {
    status: String,
    model_id: Option<String>,
    content: Option<String>,
    pages: Option<Vec<AzurePage>>,
    tables: Option<Vec<AzureTable>>,
    key_value_pairs: Option<Vec<AzureKeyValuePair>>,
    documents: Option<Vec<AzureDocument>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzurePage {
    page_number: i32,
    angle: Option<f32>,
    width: f32,
    height: f32,
    unit: String,
    words: Option<Vec<AzureWord>>,
    lines: Option<Vec<AzureLine>>,
    selection_marks: Option<Vec<AzureSelectionMark>>,
}

#[derive(Debug, Deserialize)]
struct AzureWord {
    content: String,
    polygon: Vec<[f32; 2]>,
    confidence: Option<f32>,
    span: AzureSpan,
}

#[derive(Debug, Deserialize)]
struct AzureLine {
    content: String,
    polygon: Vec<[f32; 2]>,
    spans: Vec<AzureSpan>,
}

#[derive(Debug, Deserialize)]
struct AzureSelectionMark {
    state: String,
    polygon: Vec<[f32; 2]>,
    confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct AzureSpan {
    offset: i32,
    length: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureTable {
    row_count: i32,
    column_count: i32,
    cells: Vec<AzureTableCell>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureTableCell {
    kind: Option<String>,
    row_index: i32,
    column_index: i32,
    row_span: Option<i32>,
    column_span: Option<i32>,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AzureKeyValuePair {
    key: AzureKeyValueElement,
    value: AzureKeyValueElement,
    confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct AzureKeyValueElement {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureDocument {
    doc_type: String,
    fields: HashMap<String, AzureField>,
    confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct AzureField {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_adapter_creation() {
        let config = AzureConfig {
            endpoint: "https://test.cognitiveservices.azure.com".to_string(),
            key: "test-key".to_string(),
            api_version: "2024-02-29-preview".to_string(),
        };
        
        let adapter = AzureDocumentIntelligenceAdapter::new(config);
        assert!(adapter.build_url("prebuilt-read").contains("prebuilt-read"));
    }
}

