use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    Lexical,
    Syntax,
    Semantic,
    Type,
    Lint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub machine_code: String,
    pub span: Span,
    pub category: DiagnosticCategory,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(
        message: String,
        machine_code: String,
        span: Span,
        category: DiagnosticCategory,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            message,
            machine_code,
            span,
            category,
            suggestion,
        }
    }
}
