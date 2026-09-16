use ttrpg_engine_lib::{
    advance_day, apply_event, enter_event_for_sim, legal_event_ids, new_game,
    submit_event_for_sim,
};
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn reply(mut stream: std::net::TcpStream, status: &str, body: serde_json::Value) {
    let text = body.to_string();
    let _ = write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}", text.len());
}
fn main() {
    let addr = std::env::var("GAME_API_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let dataset = std::env::var("DATASET_PATH").unwrap_or_else(|_| "dataset".into());
    let state = Arc::new(Mutex::new(new_game(dataset)));
    let listener = TcpListener::bind(&addr).expect("bind API address");
    eprintln!("game API listening on http://{addr}");
    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else { continue };
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 4096];
        let header_end = loop {
            let read = stream.read(&mut chunk).unwrap_or(0);
            if read == 0 {
                break None;
            }
            bytes.extend_from_slice(&chunk[..read]);
            if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break Some(end + 4);
            }
            if bytes.len() > 1024 * 1024 {
                break None;
            }
        };
        let Some(header_end) = header_end else {
            reply(stream, "400 Bad Request", json!({"error": "invalid HTTP request"}));
            continue;
        };
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers.lines()
            .find_map(|line| line.strip_prefix("Content-Length:").or_else(|| line.strip_prefix("content-length:")))
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut chunk).unwrap_or(0);
            if read == 0 { break; }
            bytes.extend_from_slice(&chunk[..read]);
        }
        let request = String::from_utf8_lossy(&bytes[..bytes.len().min(header_end + content_length)]);
        let mut lines = request.lines();
        let first = lines.next().unwrap_or("");
        let mut parts = first.split_whitespace();
        let method = parts.next().unwrap_or(""); let path = parts.next().unwrap_or("/");
        let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
        let mut game = state.lock().unwrap();
        let result = match (method, path) {
            ("GET", "/state") => Ok(serde_json::to_value(&*game).unwrap()),
            ("GET", "/events") => Ok(json!(legal_event_ids(&game))),
            ("POST", "/advance") => advance_day(&mut game).map(|_| json!(&*game)),
            ("POST", "/activity") => serde_json::from_str::<serde_json::Value>(body).ok()
                .and_then(|v| v.get("id").and_then(|x| x.as_str()).map(str::to_owned))
                .ok_or_else(|| "body must be {\"id\":\"event-id\"}".into())
                .and_then(|id| apply_event(&mut game, &id).map(|_| json!(&*game))),
            ("POST", "/event") => serde_json::from_str::<serde_json::Value>(body).ok()
                .and_then(|v| Some((v.get("event_id")?.as_str()?.to_owned(), v.get("object_id")?.as_str()?.to_owned())))
                .ok_or_else(|| "body must be {\"event_id\":\"...\",\"object_id\":\"...\"}".into())
                .and_then(|(event_id, object_id)| enter_event_for_sim(&mut game, &event_id, &object_id).map(|_| json!(&*game))),
            ("POST", "/event-result") => serde_json::from_str::<serde_json::Value>(body).ok()
                .and_then(|v| Some((v.get("entry_id")?.as_str()?.to_owned(), v.get("result")?.as_str()?.to_owned())))
                .ok_or_else(|| "body must be {\"entry_id\":\"...\",\"result\":\"success\"}".into())
                .and_then(|(id, result)| submit_event_for_sim(&mut game, &id, &result).map(|_| json!(&*game))),
            _ => Err("not found".into()),
        };
        match result { Ok(value) => reply(stream, "200 OK", value), Err(error) => reply(stream, "400 Bad Request", json!({"error": error})) }
    }
}
