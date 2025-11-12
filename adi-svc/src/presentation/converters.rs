/// Converters between protobuf messages and domain models
/// 
/// This module handles the conversion between gRPC protobuf messages
/// and our internal domain models.

use crate::domain::*;
use crate::generated as pb;

/// Convert protobuf AnalyzeRequest to domain AnalyzeDocumentRequest
pub fn pb_to_analyze_request(
    request: pb::AnalyzeRequest,
    model_type: ModelType,
) -> Result<AnalyzeDocumentRequest, String> {
    let source = match request.source {
        Some(pb::analyze_request::Source::DocumentUrl(url)) => {
            DocumentSource::Url(url)
        }
        Some(pb::analyze_request::Source::DocumentBytes(bytes)) => {
            DocumentSource::Bytes(bytes)
        }
        None => return Err("No document source provided".to_string()),
    };
    
    let options = request.options.map(pb_to_options).unwrap_or_default();
    
    Ok(AnalyzeDocumentRequest {
        source,
        model_type,
        options,
    })
}

/// Convert protobuf AnalyzeOptions to domain AnalyzeOptions
pub fn pb_to_options(options: pb::AnalyzeOptions) -> AnalyzeOptions {
    AnalyzeOptions {
        locale: if options.locale.is_empty() {
            None
        } else {
            Locale::new(options.locale).ok()
        },
        pages: if options.pages.is_empty() {
            None
        } else {
            PageRange::new(options.pages).ok()
        },
        features: options
            .features
            .into_iter()
            .filter_map(pb_to_feature)
            .collect(),
    }
}

/// Convert protobuf Feature to domain AnalysisFeature
pub fn pb_to_feature(feature: i32) -> Option<AnalysisFeature> {
    match pb::Feature::try_from(feature).ok()? {
        pb::Feature::OcrHighResolution => Some(AnalysisFeature::OcrHighResolution),
        pb::Feature::Languages => Some(AnalysisFeature::Languages),
        pb::Feature::Barcodes => Some(AnalysisFeature::Barcodes),
        pb::Feature::Formulas => Some(AnalysisFeature::Formulas),
        pb::Feature::StyleFont => Some(AnalysisFeature::StyleFont),
        pb::Feature::KeyValuePairs => Some(AnalysisFeature::KeyValuePairs),
        _ => None,
    }
}

/// Convert domain AnalysisOperation to protobuf AnalyzeResponse
pub fn operation_to_pb_response(
    operation: AnalysisOperation,
    result: Option<AnalysisResult>,
) -> pb::AnalyzeResponse {
    pb::AnalyzeResponse {
        status: operation_status_to_pb(operation.status) as i32,
        operation_id: operation.operation_id,
        result: result.map(result_to_pb),
        error: None,
    }
}

/// Convert domain OperationStatus to protobuf AnalysisStatus
pub fn operation_status_to_pb(status: OperationStatus) -> i32 {
    match status {
        OperationStatus::NotStarted => pb::AnalysisStatus::StatusUnspecified as i32,
        OperationStatus::Running => pb::AnalysisStatus::StatusRunning as i32,
        OperationStatus::Succeeded => pb::AnalysisStatus::StatusSucceeded as i32,
        OperationStatus::Failed => pb::AnalysisStatus::StatusFailed as i32,
        OperationStatus::Canceled => pb::AnalysisStatus::StatusFailed as i32,
    }
}

/// Convert domain AnalysisResult to protobuf AnalysisResult
pub fn result_to_pb(result: AnalysisResult) -> pb::AnalysisResult {
    pb::AnalysisResult {
        model_id: result.model_id,
        api_version: result.api_version,
        content: result.content,
        pages: result.pages.into_iter().map(page_to_pb).collect(),
        tables: result.tables.into_iter().map(table_to_pb).collect(),
        key_value_pairs: result.key_value_pairs.into_iter().map(kvp_to_pb).collect(),
        entities: vec![],
        styles: vec![],
        documents: result.documents.into_iter().map(document_to_pb).collect(),
    }
}

/// Convert domain DocumentPage to protobuf
pub fn page_to_pb(page: DocumentPage) -> pb::DocumentPage {
    pb::DocumentPage {
        page_number: page.page_number,
        angle: page.angle,
        width: page.width,
        height: page.height,
        unit: page.unit,
        spans: vec![],
        words: page.words.into_iter().map(word_to_pb).collect(),
        lines: page.lines.into_iter().map(line_to_pb).collect(),
        selection_marks: page.selection_marks.into_iter().map(selection_mark_to_pb).collect(),
        barcodes: vec![],
    }
}

pub fn word_to_pb(word: DocumentWord) -> pb::DocumentWord {
    pb::DocumentWord {
        content: word.content,
        polygon: Some(pb::BoundingPolygon {
            points: word.polygon.into_iter().map(point_to_pb).collect(),
        }),
        confidence: word.confidence,
        span: Some(span_to_pb(word.span)),
    }
}

pub fn line_to_pb(line: DocumentLine) -> pb::DocumentLine {
    pb::DocumentLine {
        content: line.content,
        polygon: Some(pb::BoundingPolygon {
            points: line.polygon.into_iter().map(point_to_pb).collect(),
        }),
        spans: line.spans.into_iter().map(span_to_pb).collect(),
    }
}

pub fn selection_mark_to_pb(mark: SelectionMark) -> pb::SelectionMark {
    pb::SelectionMark {
        state: match mark.state {
            SelectionMarkState::Selected => pb::SelectionMarkState::SelectionMarkSelected as i32,
            SelectionMarkState::Unselected => pb::SelectionMarkState::SelectionMarkUnselected as i32,
        },
        polygon: Some(pb::BoundingPolygon {
            points: mark.polygon.into_iter().map(point_to_pb).collect(),
        }),
        confidence: mark.confidence,
        span: None,
    }
}

pub fn point_to_pb(point: Point) -> pb::Point {
    pb::Point {
        x: point.x,
        y: point.y,
    }
}

pub fn span_to_pb(span: Span) -> pb::DocumentSpan {
    pb::DocumentSpan {
        offset: span.offset,
        length: span.length,
    }
}

pub fn table_to_pb(table: DocumentTable) -> pb::DocumentTable {
    pb::DocumentTable {
        row_count: table.row_count,
        column_count: table.column_count,
        cells: table.cells.into_iter().map(cell_to_pb).collect(),
        spans: vec![],
        bounding_regions: None,
    }
}

pub fn cell_to_pb(cell: TableCell) -> pb::DocumentTableCell {
    pb::DocumentTableCell {
        kind: match cell.kind {
            CellKind::Content => pb::CellKind::Content as i32,
            CellKind::RowHeader => pb::CellKind::RowHeader as i32,
            CellKind::ColumnHeader => pb::CellKind::ColumnHeader as i32,
            CellKind::StubHead => pb::CellKind::StubHead as i32,
            CellKind::Description => pb::CellKind::Description as i32,
        },
        row_index: cell.row_index,
        column_index: cell.column_index,
        row_span: cell.row_span,
        column_span: cell.column_span,
        content: cell.content,
        spans: vec![],
        bounding_regions: None,
    }
}

pub fn kvp_to_pb(kvp: KeyValuePair) -> pb::KeyValuePair {
    pb::KeyValuePair {
        key: Some(pb::DocumentField {
            r#type: pb::FieldType::String as i32,
            value_string: kvp.key,
            value_number: 0.0,
            value_integer: 0,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: kvp.confidence,
        }),
        value: Some(pb::DocumentField {
            r#type: pb::FieldType::String as i32,
            value_string: kvp.value,
            value_number: 0.0,
            value_integer: 0,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: kvp.confidence,
        }),
        confidence: kvp.confidence,
    }
}

pub fn document_to_pb(doc: ExtractedDocument) -> pb::Document {
    pb::Document {
        doc_type: doc.doc_type,
        fields: doc
            .fields
            .into_iter()
            .map(|(k, v)| (k, field_to_pb(v)))
            .collect(),
        bounding_regions: vec![],
        spans: vec![],
        confidence: doc.confidence,
    }
}

pub fn field_to_pb(field: DocumentField) -> pb::DocumentField {
    match field {
        DocumentField::String(s) => pb::DocumentField {
            r#type: pb::FieldType::String as i32,
            value_string: s,
            value_number: 0.0,
            value_integer: 0,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: 1.0,
        },
        DocumentField::Number(n) => pb::DocumentField {
            r#type: pb::FieldType::Number as i32,
            value_string: String::new(),
            value_number: n,
            value_integer: 0,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: 1.0,
        },
        DocumentField::Integer(i) => pb::DocumentField {
            r#type: pb::FieldType::Integer as i32,
            value_string: String::new(),
            value_number: 0.0,
            value_integer: i,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: 1.0,
        },
        DocumentField::Boolean(b) => pb::DocumentField {
            r#type: pb::FieldType::Boolean as i32,
            value_string: String::new(),
            value_number: 0.0,
            value_integer: 0,
            value_boolean: b,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: 1.0,
        },
        _ => pb::DocumentField {
            r#type: pb::FieldType::Unspecified as i32,
            value_string: String::new(),
            value_number: 0.0,
            value_integer: 0,
            value_boolean: false,
            value_date: None,
            value_time: None,
            value_array: vec![],
            value_object: Default::default(),
            content: String::new(),
            spans: vec![],
            bounding_regions: vec![],
            confidence: 1.0,
        },
    }
}

