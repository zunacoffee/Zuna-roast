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
use tokio_tungstenite::{connect_async, tungstenite::Message};

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
        match tokio::time::timeout(Duration::from_secs(10), connect_async(url.as_str())).await {
            Ok(Ok((stream, _))) => {
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
            Ok(Err(e)) => emit(&app, id, "error", Some(e.to_string())),
            Err(_) => emit(&app, id, "error", Some("Connect timeout".into())),
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
