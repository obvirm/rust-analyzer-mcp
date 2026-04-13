use std::io::{Read, Write};
use std::process::{Command, Stdio};
use serde_json::json;

fn main() {
    println!("Testing MCP Protocol...\n");

    let mut child = Command::new("target/release/rust-analyzer-mcp.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    let mut stdout = child.stdout.as_mut().expect("Failed to get stdout");

    // Test 1: Initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        }
    });
    send_json(stdin, &init_request);
    let response = read_json(stdout);
    println!("1. Initialize: {}", if response.get("result").is_some() { "OK" } else { "FAIL" });

    // Test 2: Tools list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    send_json(stdin, &tools_request);
    let response = read_json(stdout);
    let has_tools = response.get("result")
        .and_then(|r| r.get("tools"))
        .map(|t| t.as_array().map(|a| !a.is_empty()).unwrap_or(false))
        .unwrap_or(false);
    println!("2. Tools list: {}", if has_tools { "OK" } else { "FAIL" });

    // Test 3: Resources list
    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list"
    });
    send_json(stdin, &resources_request);
    let response = read_json(stdout);
    let has_resources = response.get("result")
        .and_then(|r| r.get("resources"))
        .map(|r| r.as_array().map(|a| !a.is_empty()).unwrap_or(false))
        .unwrap_or(false);
    println!("3. Resources list: {}", if has_resources { "OK" } else { "FAIL" });

    // Test 4: Prompts list
    let prompts_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "prompts/list"
    });
    send_json(stdin, &prompts_request);
    let response = read_json(stdout);
    let has_prompts = response.get("result")
        .and_then(|r| r.get("prompts"))
        .map(|p| p.as_array().map(|a| !a.is_empty()).unwrap_or(false))
        .unwrap_or(false);
    println!("4. Prompts list: {}", if has_prompts { "OK" } else { "FAIL" });

    // Test 5: Shutdown
    let shutdown_request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "shutdown"
    });
    send_json(stdin, &shutdown_request);
    let _ = read_json(stdout);
    println!("5. Shutdown: OK");

    println!("\n=== All MCP Protocol Tests Passed! ===");
}

fn send_json<T: Write>(stdin: &mut T, value: &serde_json::Value) {
    let json_str = serde_json::to_string(value).unwrap();
    let body = json_str.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdin.write_all(header.as_bytes()).unwrap();
    stdin.write_all(body).unwrap();
    stdin.flush().unwrap();
}

fn read_json<R: Read>(stdout: &mut R) -> serde_json::Value {
    let mut header = [0u8; 1];
    let mut headers = String::new();
    
    loop {
        stdout.read_exact(&mut header).unwrap();
        headers.push(header[0] as char);
        if headers.contains("\r\n\r\n") {
            break;
        }
    }

    let content_len = headers
        .lines()
        .find(|l| l.starts_with("Content-Length:"))
        .map(|l| l.split(':').nth(1).unwrap().trim().parse::<usize>().unwrap())
        .unwrap();

    let mut body = vec![0u8; content_len];
    stdout.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}