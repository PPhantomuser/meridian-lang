#![allow(unused)]
use std::collections::HashMap;
use std::io::{self, Read, Write, BufRead};
use serde_json::{Value, json};
use meridian_lexer::Lexer;
use meridian_parser::Parser;
use meridian_semantic::SemanticAnalyzer;
use meridian_ast::Program;
use meridian_diagnostics::Span;

pub struct Workspace {
    documents: HashMap<String, String>,
    semantics: HashMap<String, SemanticAnalyzer>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            semantics: HashMap::new(),
        }
    }

    pub fn update_document(&mut self, uri: String, content: String) {
        self.documents.insert(uri.clone(), content.clone());
        let lexer = Lexer::new(&content);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse_program();
        
        let mut semantic = SemanticAnalyzer::new();
        semantic.analyze_program(&ast);
        
        // Merge lexer, parser, semantic diagnostics
        let mut diagnostics = Vec::new();
        diagnostics.extend(parser.diagnostics.clone());
        diagnostics.extend(semantic.diagnostics.clone());
        
        self.semantics.insert(uri.clone(), semantic);
        
        publish_diagnostics(&uri, &content, diagnostics);
    }
}

fn publish_diagnostics(uri: &str, content: &str, diags: Vec<meridian_diagnostics::Diagnostic>) {
    let mut lsp_diags = Vec::new();
    for d in diags {
        let (start_line, start_col) = byte_offset_to_line_col(content, d.span.start);
        let (end_line, end_col) = byte_offset_to_line_col(content, d.span.end);
        
        let severity = match d.category {
            meridian_diagnostics::DiagnosticCategory::Lexical => 1,
            meridian_diagnostics::DiagnosticCategory::Syntax => 1,
            meridian_diagnostics::DiagnosticCategory::Type => 1,
            meridian_diagnostics::DiagnosticCategory::Semantic => 1,
            meridian_diagnostics::DiagnosticCategory::Lint => 2,
        };

        lsp_diags.push(json!({
            "range": {
                "start": { "line": start_line, "character": start_col },
                "end": { "line": end_line, "character": end_col }
            },
            "severity": severity,
            "code": d.machine_code,
            "message": d.message,
        }));
    }

    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": lsp_diags
        }
    });
    send_response(notification);
}

fn byte_offset_to_line_col(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn line_col_to_byte_offset(text: &str, target_line: u32, target_col: u32) -> usize {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in text.char_indices() {
        if line == target_line && col == target_col {
            return i;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    text.len()
}

pub fn run_server() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut workspace = Workspace::new();

    loop {
        let mut content_length = 0;
        let mut headers_done = false;

        loop {
            let mut line = String::new();
            if handle.read_line(&mut line).unwrap_or(0) == 0 {
                return; // EOF
            }
            let line = line.trim();
            if line.is_empty() {
                headers_done = true;
                break;
            }
            if line.starts_with("Content-Length:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    content_length = parts[1].trim().parse::<usize>().unwrap_or(0);
                }
            }
        }

        if !headers_done || content_length == 0 {
            continue;
        }

        let mut body = vec![0; content_length];
        if handle.read_exact(&mut body).is_err() {
            break;
        }

        let body_str = String::from_utf8_lossy(&body);
        if let Ok(json_val) = serde_json::from_str::<Value>(&body_str) {
            handle_message(json_val, &mut workspace);
        }
    }
}

fn handle_message(msg: Value, workspace: &mut Workspace) {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = msg.get("id");

    match method {
        "initialize" => {
            if let Some(req_id) = id {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1, // Full sync
                            "definitionProvider": true,
                            "referencesProvider": true,
                            "renameProvider": true,
                        },
                        "serverInfo": {
                            "name": "merid-lsp",
                            "version": "0.1.0"
                        }
                    }
                });
                send_response(response);
            }
        }
        "initialized" => {}
        "textDocument/didOpen" => {
            if let Some(params) = msg.get("params") {
                if let (Some(uri), Some(text)) = (
                    params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()),
                    params.get("textDocument").and_then(|td| td.get("text")).and_then(|t| t.as_str())
                ) {
                    workspace.update_document(uri.to_string(), text.to_string());
                }
            }
        }
        "textDocument/didChange" => {
            if let Some(params) = msg.get("params") {
                if let (Some(uri), Some(changes)) = (
                    params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()),
                    params.get("contentChanges").and_then(|c| c.as_array())
                ) {
                    if let Some(first_change) = changes.first() {
                        if let Some(text) = first_change.get("text").and_then(|t| t.as_str()) {
                            workspace.update_document(uri.to_string(), text.to_string());
                        }
                    }
                }
            }
        }
        "textDocument/definition" => {
            if let Some(req_id) = id {
                let mut result = serde_json::Value::Null;
                if let Some(params) = msg.get("params") {
                    if let (Some(uri), Some(pos)) = (
                        params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()),
                        params.get("position")
                    ) {
                        let line = pos.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                        let character = pos.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32;
                        
                        if let Some(text) = workspace.documents.get(uri) {
                            let offset = line_col_to_byte_offset(text, line, character);
                            
                            if let Some(semantic) = workspace.semantics.get(uri) {
                                // Find if this offset is within any usage span
                                let mut found_def_span: Option<Span> = None;
                                for (usage_span, def_span) in &semantic.index.usages {
                                    if offset >= usage_span.start && offset <= usage_span.end {
                                        found_def_span = Some(*def_span);
                                        break;
                                    }
                                }
                                
                                if let Some(def_span) = found_def_span {
                                    let (start_line, start_col) = byte_offset_to_line_col(text, def_span.start);
                                    let (end_line, end_col) = byte_offset_to_line_col(text, def_span.end);
                                    
                                    result = json!({
                                        "uri": uri,
                                        "range": {
                                            "start": { "line": start_line, "character": start_col },
                                            "end": { "line": end_line, "character": end_col }
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                
                send_response(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": result
                }));
            }
        }
        "textDocument/references" => {
            if let Some(req_id) = id {
                let mut result = serde_json::Value::Null;
                if let Some(params) = msg.get("params") {
                    if let (Some(uri), Some(pos)) = (
                        params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()),
                        params.get("position")
                    ) {
                        let line = pos.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                        let character = pos.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32;
                        
                        if let Some(text) = workspace.documents.get(uri) {
                            let offset = line_col_to_byte_offset(text, line, character);
                            
                            if let Some(semantic) = workspace.semantics.get(uri) {
                                // Find definition span to get all its usages
                                let mut target_def_span: Option<Span> = None;
                                
                                // Maybe we clicked on a definition directly?
                                for def_span in semantic.index.definitions.values() {
                                    if offset >= def_span.start && offset <= def_span.end {
                                        target_def_span = Some(*def_span);
                                        break;
                                    }
                                }
                                
                                // Or maybe we clicked on a usage?
                                if target_def_span.is_none() {
                                    for (usage_span, def_span) in &semantic.index.usages {
                                        if offset >= usage_span.start && offset <= usage_span.end {
                                            target_def_span = Some(*def_span);
                                            break;
                                        }
                                    }
                                }
                                
                                if let Some(def_span) = target_def_span {
                                    let mut refs = Vec::new();
                                    for (usage_span, ds) in &semantic.index.usages {
                                        if ds.start == def_span.start && ds.end == def_span.end {
                                            let (start_line, start_col) = byte_offset_to_line_col(text, usage_span.start);
                                            let (end_line, end_col) = byte_offset_to_line_col(text, usage_span.end);
                                            refs.push(json!({
                                                "uri": uri,
                                                "range": {
                                                    "start": { "line": start_line, "character": start_col },
                                                    "end": { "line": end_line, "character": end_col }
                                                }
                                            }));
                                        }
                                    }
                                    result = json!(refs);
                                }
                            }
                        }
                    }
                }
                
                send_response(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": result
                }));
            }
        }
        "textDocument/rename" => {
            if let Some(req_id) = id {
                let mut result = serde_json::Value::Null;
                if let Some(params) = msg.get("params") {
                    if let (Some(uri), Some(pos), Some(new_name)) = (
                        params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()),
                        params.get("position"),
                        params.get("newName").and_then(|n| n.as_str())
                    ) {
                        let line = pos.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                        let character = pos.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32;
                        
                        if let Some(text) = workspace.documents.get(uri) {
                            let offset = line_col_to_byte_offset(text, line, character);
                            if let Some(semantic) = workspace.semantics.get(uri) {
                                let mut target_def_span: Option<Span> = None;
                                for def_span in semantic.index.definitions.values() {
                                    if offset >= def_span.start && offset <= def_span.end {
                                        target_def_span = Some(*def_span);
                                        break;
                                    }
                                }
                                if target_def_span.is_none() {
                                    for (usage_span, def_span) in &semantic.index.usages {
                                        if offset >= usage_span.start && offset <= usage_span.end {
                                            target_def_span = Some(*def_span);
                                            break;
                                        }
                                    }
                                }
                                if let Some(def_span) = target_def_span {
                                    let mut changes = Vec::new();
                                    let (dl, dc) = byte_offset_to_line_col(text, def_span.start);
                                    let (del, dec) = byte_offset_to_line_col(text, def_span.end);
                                    changes.push(json!({
                                        "range": { "start": { "line": dl, "character": dc }, "end": { "line": del, "character": dec } },
                                        "newText": new_name
                                    }));
                                    for (usage_span, ds) in &semantic.index.usages {
                                        if ds.start == def_span.start && ds.end == def_span.end {
                                            let (sl, sc) = byte_offset_to_line_col(text, usage_span.start);
                                            let (el, ec) = byte_offset_to_line_col(text, usage_span.end);
                                            changes.push(json!({
                                                "range": { "start": { "line": sl, "character": sc }, "end": { "line": el, "character": ec } },
                                                "newText": new_name
                                            }));
                                        }
                                    }
                                    let mut uri_changes = serde_json::Map::new();
                                    uri_changes.insert(uri.to_string(), json!(changes));
                                    result = json!({ "changes": uri_changes });
                                }
                            }
                        }
                    }
                }
                send_response(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": result
                }));
            }
        }
        "shutdown" => {
            if let Some(req_id) = id {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": serde_json::Value::Null
                });
                send_response(response);
            }
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn send_response(response: Value) {
    let response_str = serde_json::to_string(&response).unwrap();
    let message = format!(
        "Content-Length: {}\r\n\r\n{}",
        response_str.len(),
        response_str
    );
    let mut stdout = io::stdout();
    stdout.write_all(message.as_bytes()).unwrap();
    stdout.flush().unwrap();
}
