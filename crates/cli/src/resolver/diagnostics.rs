#![allow(unused)]
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ResolverDiagnostic {
    DependencyConflict {
        package: String,
        required_by: Vec<String>,
        conflict_ranges: Vec<String>,
        ai_hint: String,
    },
    FetchError {
        package: String,
        source: String,
        error: String,
    },
    ChecksumMismatch {
        package: String,
        expected: String,
        actual: String,
    }
}
