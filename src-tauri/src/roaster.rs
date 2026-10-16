//! WebSocket link to the roaster, run by the app itself rather than the web view.
//!
//! The screen talks to it through a small `WebSocket` stand-in (app/native.js), so the
//! roast code is the same as in the browser version. Doing the socket natively avoids
//! macOS web-view rules about plain `ws://` connections, and keeps the link alive even
//! if the window is in the background.

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest, handshake::client::Request, http::HeaderValue, Message};
use tokio_tungstenite::connect_async;

/// Ways of opening the connection, tried in order until the roaster accepts one.
/// Embedded web servers differ in what they insist on, so we start with what a web
/// browser (and Artisan's older library) sends, then fall back.
const VARIANTS: [(&str, bool, bool); 4] = [
    // (description, browser-style headers, lowercase path)
    ("browser-style", true, false),
    ("browser-style, lowercase path", true, true),
    ("plain", false, false),
    ("plain, lowercase path", false, true),
];

/// Which variant worked last time, so reconnects go straight to it.
pub static PREFERRED: AtomicUsize = AtomicUsize::new(0);

fn build_request(url: &str, browser: bool, lower: bool) -> Result<Request, String> {
    let mut url = url.to_string();
    if lower {
        if let Some(i) = url.find("://").and_then(|i| url[i + 3..].find('/').map(|j| i + 3 + j)) {
            let (a, b) = url.split_at(i);
            url = format!("{}{}", a, b.to_lowercase());
        }
    }
    let mut req = url.as_str().into_client_request().map_err(|e| e.to_string())?;
    if browser {
        let uri = req.uri().clone();
        let host = uri.host().unwrap_or_default().to_string();
        let port = uri.port_u16();
        let secure = uri.scheme_str() == Some("wss");
        let default_port = if secure { 443 } else { 80 };
        let host_hdr = match port { Some(p) if p != default_port => format!("{}:{}", host, p), _ => host.clone() };
        let origin = format!("{}://{}", if secure { "https" } else { "http" }, host_hdr);
        let h = req.headers_mut();
        h.insert("Host", HeaderValue::from_str(&host_hdr).map_err(|e| e.to_string())?);
        h.insert("Origin", HeaderValue::from_str(&origin).map_err(|e| e.to_string())?);
        h.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) ZunaRoast"));
        h.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        h.insert("Pragma", HeaderValue::from_static("no-cache"));
    }
    Ok(req)
}

fn describe(e: &tungstenite::Error) -> String {
    match e {
        tungstenite::Error::Http(resp) => {
            let body = resp.body().as_ref().map(|b| String::from_utf8_lossy(b).trim().chars().take(200).collect::<String>()).unwrap_or_default();
            if body.is_empty() { format!("roaster answered HTTP {}", resp.status()) } else { format!("roaster answered HTTP {}: {}", resp.status(), body) }
        }
        other => other.to_string(),
    }
}

pub enum Cmd {
    Send(String),
    Close,
}

#[derive(Default)]
pub struct Sockets(pub Mutex<HashMap<u32, mpsc::UnboundedSender<Cmd>>>);

#[derive(Clone, Serialize)]
struct WsEvent {
    id: u32,
    kind: &'static str,
    data: Option<String>,
}

fn emit(app: &AppHandle, id: u32, kind: &'static str, data: Option<String>) {
    let _ = app.emit("roaster-ws", WsEvent { id, kind, data });
}

#[tauri::command]
pub fn ws_open(app: AppHandle, state: State<Sockets>, id: u32, url: String) -> Result<(), String> {
    if !(url.starts_with("ws://") || url.starts_with("wss://")) {
        return Err("Roaster address must start with ws://".into());
    }
    let (tx, mut rx) = mpsc::unbounded_channel::<Cmd>();
    state.0.lock().map_err(|_| "lock")?.insert(id, tx);

    tauri::async_runtime::spawn(async move {
        // Try the handshake styles, starting with the one that worked last time.
        let start = PREFERRED.load(Ordering::Relaxed) % VARIANTS.len();
        let mut connected = None;
        let mut last_err = String::from("Connect failed");
        for k in 0..VARIANTS.len() {
            let v = (start + k) % VARIANTS.len();
            let (name, browser, lower) = VARIANTS[v];
            if lower && !url.split("://").nth(1).map(|r| r.chars().any(|c| c.is_ascii_uppercase())).unwrap_or(false) { continue; }
            let req = match build_request(&url, browser, lower) { Ok(r) => r, Err(e) => { last_err = e; continue; } };
            match tokio::time::timeout(Duration::from_secs(10), connect_async(req)).await {
                Ok(Ok((stream, _))) => {
                    if v != start || k > 0 { emit(&app, id, "info", Some(format!("Connected using the {} handshake", name))); }
                    PREFERRED.store(v, Ordering::Relaxed);
                    connected = Some(stream);
                    break;
                }
                Ok(Err(e)) => {
                    last_err = describe(&e);
                    emit(&app, id, "info", Some(format!("Handshake ({}) refused — {}", name, last_err)));
                    // Only an HTTP-level refusal is worth retrying another style; network errors are not.
                    if !matches!(e, tungstenite::Error::Http(_)) { break; }
                }
                Err(_) => { last_err = "Connect timeout".into(); break; }
            }
        }
        match connected {
            Some(stream) => {
                emit(&app, id, "open", None);
                let (mut write, mut read) = stream.split();
                loop {
                    tokio::select! {
                        cmd = rx.recv() => match cmd {
                            Some(Cmd::Send(text)) => {
                                if write.send(Message::Text(text.into())).await.is_err() { break; }
                            }
                            Some(Cmd::Close) | None => { let _ = write.close().await; break; }
                        },
                        msg = read.next() => match msg {
                            Some(Ok(Message::Text(t))) => emit(&app, id, "message", Some(t.to_string())),
                            Some(Ok(Message::Binary(b))) => emit(&app, id, "message", Some(String::from_utf8_lossy(&b).into_owned())),
                            Some(Ok(Message::Ping(p))) => { let _ = write.send(Message::Pong(p)).await; }
                            Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                            _ => {}
                        }
                    }
                }
            }
            None => emit(&app, id, "error", Some(last_err)),
        }
        if let Ok(mut m) = app.state::<Sockets>().0.lock() {
            m.remove(&id);
        }
        emit(&app, id, "close", None);
    });
    Ok(())
}

#[tauri::command]
pub fn ws_send(state: State<Sockets>, id: u32, text: String) -> Result<(), String> {
    let m = state.0.lock().map_err(|_| "lock")?;
    let tx = m.get(&id).ok_or("Not connected")?;
    tx.send(Cmd::Send(text)).map_err(|_| "Not connected".to_string())
}

#[tauri::command]
pub fn ws_close(state: State<Sockets>, id: u32) -> Result<(), String> {
    if let Some(tx) = state.0.lock().map_err(|_| "lock")?.get(&id) {
        let _ = tx.send(Cmd::Close);
    }
    Ok(())
}
