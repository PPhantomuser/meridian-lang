#![allow(unused)]
use std::io::{self, Read, Write, BufRead};
use serde_json::{Value, json};

fn main() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

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
            handle_message(json_val);
        }
    }
}

fn handle_message(msg: Value) {
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
                            "hoverProvider": false,
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": []
                            }
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
        "initialized" => {
            // Nothing to do for initialized notification
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
        _ => {
            // Ignore other messages for now
        }
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
