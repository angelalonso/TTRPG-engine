use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use ttrpg_engine_lib::{
    advance_day, apply_event, enter_event_for_sim, legal_event_ids, new_game, submit_event_for_sim,
};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;

fn reply(mut stream: std::net::TcpStream, status: &str, body: serde_json::Value) {
    let text = body.to_string();
    if let Err(error) = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        text.len()
    ) {
        eprintln!("failed to write API response: {error}");
    }
}

fn json_fields(body: &str, names: &[&str], usage: &str) -> Result<Vec<String>, String> {
    let value = serde_json::from_str::<serde_json::Value>(body).map_err(|_| usage.to_owned())?;
    names
        .iter()
        .map(|name| {
            value
                .get(name)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| usage.to_owned())
        })
        .collect()
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
            if bytes.len() > MAX_HEADER_BYTES {
                break None;
            }
        };
        let Some(header_end) = header_end else {
            reply(
                stream,
                "400 Bad Request",
                json!({"error": "invalid HTTP request"}),
            );
            continue;
        };
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length:")
                    .or_else(|| line.strip_prefix("content-length:"))
            })
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if content_length > MAX_BODY_BYTES {
            reply(
                stream,
                "413 Payload Too Large",
                json!({"error": "request body is too large"}),
            );
            continue;
        }
        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut chunk).unwrap_or(0);
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..read]);
        }
        let request =
            String::from_utf8_lossy(&bytes[..bytes.len().min(header_end + content_length)]);
        let mut lines = request.lines();
        let first = lines.next().unwrap_or("");
        let mut parts = first.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");
        let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
        let mut game = match state.lock() {
            Ok(game) => game,
            Err(error) => {
                reply(
                    stream,
                    "500 Internal Server Error",
                    json!({"error": format!("game state lock failed: {error}")}),
                );
                continue;
            }
        };
        let result = match (method, path) {
            ("GET", "/health") => Ok(json!({"status": "ok"})),
            ("GET", "/state") => serde_json::to_value(&*game).map_err(|error| error.to_string()),
            ("GET", "/events") => Ok(json!(legal_event_ids(&game))),
            ("POST", "/advance") => advance_day(&mut game).map(|_| json!(&*game)),
            ("POST", "/activity") => {
                json_fields(body, &["id"], "body must be {\"id\":\"event-id\"}")
                    .map(|fields| fields[0].clone())
                    .and_then(|id| apply_event(&mut game, &id).map(|_| json!(&*game)))
            }
            ("POST", "/event") => json_fields(
                body,
                &["event_id", "object_id"],
                "body must be {\"event_id\":\"...\",\"object_id\":\"...\"}",
            )
            .and_then(|fields| {
                enter_event_for_sim(&mut game, &fields[0], &fields[1]).map(|_| json!(&*game))
            }),
            ("POST", "/event-result") => json_fields(
                body,
                &["entry_id", "result"],
                "body must be {\"entry_id\":\"...\",\"result\":\"success\"}",
            )
            .and_then(|fields| {
                submit_event_for_sim(&mut game, &fields[0], &fields[1]).map(|_| json!(&*game))
            }),
            _ => Err("not found".into()),
        };
        match result {
            Ok(value) => reply(stream, "200 OK", value),
            Err(error) if error == "not found" => {
                reply(stream, "404 Not Found", json!({"error": error}));
            }
            Err(error) => reply(stream, "400 Bad Request", json!({"error": error})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::json_fields;

    #[test]
    fn parses_required_json_string_fields() {
        assert_eq!(
            json_fields(
                r#"{"event_id":"race","object_id":"car"}"#,
                &["event_id", "object_id"],
                "invalid",
            )
            .unwrap(),
            vec!["race", "car"]
        );
    }

    #[test]
    fn rejects_invalid_or_non_string_fields() {
        assert!(json_fields(r#"{"id":42}"#, &["id"], "invalid").is_err());
        assert!(json_fields("not-json", &["id"], "invalid").is_err());
    }
}
