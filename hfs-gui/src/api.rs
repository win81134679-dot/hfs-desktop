//! HFS v3 管理 API。只連 127.0.0.1，靠 localhost_admin 免密碼。

use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::time::Duration;

fn url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{port}/~/api/{path}")
}

pub fn call(port: u16, name: &str, body: &Value) -> Result<Value, String> {
    call_timeout(port, name, body, Duration::from_secs(10))
}

pub fn get_status(port: u16) -> Result<Value, String> {
    call_timeout(port, "get_status", &Value::Object(Default::default()), Duration::from_secs(2))
}

pub fn call_timeout(port: u16, name: &str, body: &Value, timeout: Duration) -> Result<Value, String> {
    let res = ureq::post(&url(port, name))
        .timeout(timeout)
        .set("x-hfs-anti-csrf", "1")
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());
    match res {
        Ok(resp) => {
            let text = resp.into_string().map_err(|e| e.to_string())?;
            if text.trim().is_empty() {
                Ok(Value::Null)
            } else {
                serde_json::from_str(&text).map_err(|e| e.to_string())
            }
        }
        Err(ureq::Error::Status(code, resp)) => {
            Err(format!("HTTP {code} {}", resp.into_string().unwrap_or_default()))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 讀 SSE 串流，每則 `data:` 解析成 JSON 交給 on_data；on_data 回傳 false 就結束
pub fn sse(port: u16, path: &str, mut on_data: impl FnMut(Value) -> bool) -> Result<(), String> {
    let resp = ureq::get(&url(port, path))
        .set("Accept", "text/event-stream")
        .set("x-hfs-anti-csrf", "1")
        .call()
        .map_err(|e| e.to_string())?;
    let reader = BufReader::new(resp.into_reader());
    let mut data = String::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            if !data.is_empty() {
                if let Ok(v) = serde_json::from_str(&data) {
                    if !on_data(v) {
                        return Ok(());
                    }
                }
                data.clear();
            }
        } else if let Some(d) = line.strip_prefix("data:") {
            data.push_str(d.strip_prefix(' ').unwrap_or(d));
        }
    }
    Ok(())
}
