use std::sync::Arc;
use std::time::Duration;

use flate2::read::ZlibDecoder;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMsg};

use crate::discord_types::*;

const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json&compress=zlib-stream";

/// Events emitted by the gateway connection to the bridge core
#[derive(Debug)]
pub enum DiscordEvent {
    Ready(DiscordReady),
    GuildCreate(DiscordGuild),
    MessageCreate(DiscordMessage),
    MessageUpdate(DiscordMessage),
    MessageDelete(MessageDelete),
    TypingStart(TypingStart),
    ReactionAdd(MessageReactionUpdate),
    ReactionRemove(MessageReactionUpdate),
    PresenceUpdate(serde_json::Value),
    ChannelCreate(DiscordChannel),
    ChannelUpdate(DiscordChannel),
    ChannelDelete(DiscordChannel),
    GuildMemberAdd { guild_id: String, member: DiscordGuildMember },
    GuildMemberRemove { guild_id: String, user: DiscordUser },
    Reconnect,
}

/// Manages the Discord gateway WebSocket connection
pub struct DiscordGateway {
    token: String,
    event_tx: mpsc::UnboundedSender<DiscordEvent>,
}

impl DiscordGateway {
    pub fn new(token: String, event_tx: mpsc::UnboundedSender<DiscordEvent>) -> Self {
        Self { token, event_tx }
    }

    /// Connect and run the gateway loop. Reconnects automatically.
    pub async fn run(&self) {
        let mut resume_url: Option<String> = None;
        let mut session_id: Option<String> = None;
        let mut seq: Option<u64> = None;

        loop {
            let url = resume_url.as_deref().unwrap_or(GATEWAY_URL);
            tracing::info!("Connecting to Discord gateway: {url}");

            match self.run_connection(url, &session_id, &seq).await {
                ConnectionResult::Reconnect { new_seq, new_session_id, new_resume_url } => {
                    seq = new_seq.or(seq);
                    if let Some(s) = new_session_id {
                        session_id = Some(s);
                    }
                    if let Some(u) = new_resume_url {
                        resume_url = Some(u);
                    }
                    tracing::info!("Reconnecting in 2s (seq={seq:?}, session={session_id:?})");
                    time::sleep(Duration::from_secs(2)).await;
                }
                ConnectionResult::InvalidSession => {
                    tracing::warn!("Invalid session, starting fresh in 5s");
                    session_id = None;
                    seq = None;
                    resume_url = None;
                    time::sleep(Duration::from_secs(5)).await;
                }
                ConnectionResult::Fatal(msg) => {
                    tracing::error!("Fatal gateway error: {msg}");
                    time::sleep(Duration::from_secs(30)).await;
                    // Reset everything
                    session_id = None;
                    seq = None;
                    resume_url = None;
                }
            }
        }
    }

    async fn run_connection(
        &self,
        url: &str,
        session_id: &Option<String>,
        seq: &Option<u64>,
    ) -> ConnectionResult {
        let (ws, _) = match connect_async(url).await {
            Ok(c) => c,
            Err(e) => {
                return ConnectionResult::Fatal(format!("WebSocket connect failed: {e}"));
            }
        };

        let (mut ws_tx, mut ws_rx) = ws.split();
        let mut zlib_buf = Vec::new();
        let mut heartbeat_interval_ms: u64 = 41250;
        let mut current_seq: Option<u64> = *seq;
        let mut current_session: Option<String> = session_id.clone();
        let mut current_resume_url: Option<String> = None;
        let mut heartbeat_acked = Arc::new(tokio::sync::Mutex::new(true));
        let mut identified = session_id.is_some();

        loop {
            tokio::select! {
                msg = ws_rx.next() => {
                    match msg {
                        Some(Ok(WsMsg::Binary(data))) => {
                            zlib_buf.extend_from_slice(&data);

                            // Discord zlib-stream: full message ends with 0x00 0x00 0xFF 0xFF
                            if zlib_buf.len() >= 4
                                && zlib_buf[zlib_buf.len()-4..] == [0x00, 0x00, 0xFF, 0xFF]
                            {
                                let mut decoder = ZlibDecoder::new(&zlib_buf[..]);
                                let mut json_str = String::new();
                                if let Err(e) = std::io::Read::read_to_string(&mut decoder, &mut json_str) {
                                    tracing::error!("Zlib decode error: {e}");
                                    zlib_buf.clear();
                                    continue;
                                }
                                zlib_buf.clear();

                                let payload: GatewayPayload = match serde_json::from_str(&json_str) {
                                    Ok(p) => p,
                                    Err(e) => {
                                        tracing::error!("JSON parse error: {e}");
                                        continue;
                                    }
                                };

                                if let Some(s) = payload.s {
                                    current_seq = Some(s);
                                }

                                match payload.op {
                                    Opcode::Hello => {
                                        if let Some(d) = &payload.d {
                                            heartbeat_interval_ms = d["heartbeat_interval"].as_u64().unwrap_or(41250);
                                        }
                                        tracing::info!("Hello, heartbeat interval: {heartbeat_interval_ms}ms");

                                        // Start heartbeat task
                                        let interval = heartbeat_interval_ms;
                                        let acked = heartbeat_acked.clone();
                                        let (hb_tx, mut hb_rx) = mpsc::channel::<String>(1);

                                        // We can't easily send from a spawned task through ws_tx,
                                        // so we'll handle heartbeat in the select! below via interval tick

                                        // Send identify or resume
                                        if let Some(sid) = &current_session {
                                            let resume = json!({
                                                "op": Opcode::Resume as u8,
                                                "d": {
                                                    "token": self.token,
                                                    "session_id": sid,
                                                    "seq": current_seq,
                                                }
                                            });
                                            let _ = ws_tx.send(WsMsg::Text(resume.to_string().into())).await;
                                            tracing::info!("Sent Resume");
                                        } else {
                                            let identify = json!({
                                                "op": Opcode::Identify as u8,
                                                "d": {
                                                    "token": self.token,
                                                    "intents": BRIDGE_INTENTS,
                                                    "properties": {
                                                        "os": "linux",
                                                        "browser": "rift-bridge",
                                                        "device": "rift-bridge"
                                                    },
                                                    "compress": false,
                                                    "large_threshold": 250,
                                                }
                                            });
                                            let _ = ws_tx.send(WsMsg::Text(identify.to_string().into())).await;
                                            identified = true;
                                            tracing::info!("Sent Identify");
                                        }

                                        // Start heartbeat loop
                                        let acked2 = heartbeat_acked.clone();
                                        // Heartbeat is handled below via tokio interval
                                    }
                                    Opcode::HeartbeatAck => {
                                        *heartbeat_acked.lock().await = true;
                                    }
                                    Opcode::Heartbeat => {
                                        // Server requested immediate heartbeat
                                        let hb = json!({ "op": 1, "d": current_seq });
                                        let _ = ws_tx.send(WsMsg::Text(hb.to_string().into())).await;
                                    }
                                    Opcode::Reconnect => {
                                        tracing::info!("Server requested reconnect");
                                        let _ = self.event_tx.send(DiscordEvent::Reconnect);
                                        return ConnectionResult::Reconnect {
                                            new_seq: current_seq,
                                            new_session_id: current_session,
                                            new_resume_url: current_resume_url,
                                        };
                                    }
                                    Opcode::InvalidSession => {
                                        let resumable = payload.d.and_then(|v| v.as_bool()).unwrap_or(false);
                                        if resumable {
                                            return ConnectionResult::Reconnect {
                                                new_seq: current_seq,
                                                new_session_id: current_session,
                                                new_resume_url: current_resume_url,
                                            };
                                        } else {
                                            return ConnectionResult::InvalidSession;
                                        }
                                    }
                                    Opcode::Dispatch => {
                                        if let Some(event_name) = &payload.t {
                                            self.handle_dispatch(event_name, payload.d).await;
                                            // Extract session info from READY
                                            if event_name == "READY" {
                                                // Already handled in handle_dispatch, but grab session_id
                                                // We need to parse again... or we stored it in handle_dispatch
                                                // Let's peek at the json_str we already have
                                                if let Ok(full) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                                    if let Some(d) = full.get("d") {
                                                        current_session = d.get("session_id")
                                                            .and_then(|v| v.as_str())
                                                            .map(String::from);
                                                        current_resume_url = d.get("resume_gateway_url")
                                                            .and_then(|v| v.as_str())
                                                            .map(|u| format!("{u}/?v=10&encoding=json&compress=zlib-stream"));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(Ok(WsMsg::Close(frame))) => {
                            let code = frame.as_ref().map(|f| f.code);
                            tracing::warn!("WebSocket closed: {code:?}");
                            match code {
                                Some(c) if c == tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(4004) => {
                                    return ConnectionResult::Fatal("Authentication failed (4004)".into());
                                }
                                _ => {
                                    return ConnectionResult::Reconnect {
                                        new_seq: current_seq,
                                        new_session_id: current_session,
                                        new_resume_url: current_resume_url,
                                    };
                                }
                            }
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {e}");
                            return ConnectionResult::Reconnect {
                                new_seq: current_seq,
                                new_session_id: current_session,
                                new_resume_url: current_resume_url,
                            };
                        }
                        None => {
                            return ConnectionResult::Reconnect {
                                new_seq: current_seq,
                                new_session_id: current_session,
                                new_resume_url: current_resume_url,
                            };
                        }
                        _ => {}
                    }
                }
                // Heartbeat tick
                _ = time::sleep(Duration::from_millis(heartbeat_interval_ms)) => {
                    let acked = *heartbeat_acked.lock().await;
                    if !acked {
                        tracing::warn!("Heartbeat not acked, reconnecting");
                        return ConnectionResult::Reconnect {
                            new_seq: current_seq,
                            new_session_id: current_session,
                            new_resume_url: current_resume_url,
                        };
                    }
                    *heartbeat_acked.lock().await = false;
                    let hb = json!({ "op": 1, "d": current_seq });
                    if ws_tx.send(WsMsg::Text(hb.to_string().into())).await.is_err() {
                        return ConnectionResult::Reconnect {
                            new_seq: current_seq,
                            new_session_id: current_session,
                            new_resume_url: current_resume_url,
                        };
                    }
                }
            }
        }
    }

    async fn handle_dispatch(&self, event: &str, data: Option<serde_json::Value>) {
        let Some(d) = data else { return };

        let result = match event {
            "READY" => serde_json::from_value::<DiscordReady>(d)
                .map(DiscordEvent::Ready)
                .map_err(|e| format!("READY parse: {e}")),

            "GUILD_CREATE" => serde_json::from_value::<DiscordGuild>(d)
                .map(DiscordEvent::GuildCreate)
                .map_err(|e| format!("GUILD_CREATE parse: {e}")),

            "MESSAGE_CREATE" => serde_json::from_value::<DiscordMessage>(d)
                .map(DiscordEvent::MessageCreate)
                .map_err(|e| format!("MESSAGE_CREATE parse: {e}")),

            "MESSAGE_UPDATE" => serde_json::from_value::<DiscordMessage>(d)
                .map(DiscordEvent::MessageUpdate)
                .map_err(|e| format!("MESSAGE_UPDATE parse: {e}")),

            "MESSAGE_DELETE" => serde_json::from_value::<MessageDelete>(d)
                .map(DiscordEvent::MessageDelete)
                .map_err(|e| format!("MESSAGE_DELETE parse: {e}")),

            "TYPING_START" => serde_json::from_value::<TypingStart>(d)
                .map(DiscordEvent::TypingStart)
                .map_err(|e| format!("TYPING_START parse: {e}")),

            "MESSAGE_REACTION_ADD" => serde_json::from_value::<MessageReactionUpdate>(d)
                .map(DiscordEvent::ReactionAdd)
                .map_err(|e| format!("REACTION_ADD parse: {e}")),

            "MESSAGE_REACTION_REMOVE" => serde_json::from_value::<MessageReactionUpdate>(d)
                .map(DiscordEvent::ReactionRemove)
                .map_err(|e| format!("REACTION_REMOVE parse: {e}")),

            "CHANNEL_CREATE" => serde_json::from_value::<DiscordChannel>(d)
                .map(DiscordEvent::ChannelCreate)
                .map_err(|e| format!("CHANNEL_CREATE parse: {e}")),

            "CHANNEL_UPDATE" => serde_json::from_value::<DiscordChannel>(d)
                .map(DiscordEvent::ChannelUpdate)
                .map_err(|e| format!("CHANNEL_UPDATE parse: {e}")),

            "CHANNEL_DELETE" => serde_json::from_value::<DiscordChannel>(d)
                .map(DiscordEvent::ChannelDelete)
                .map_err(|e| format!("CHANNEL_DELETE parse: {e}")),

            "GUILD_MEMBER_ADD" => {
                let guild_id = d.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                serde_json::from_value::<DiscordGuildMember>(d)
                    .map(|member| DiscordEvent::GuildMemberAdd { guild_id, member })
                    .map_err(|e| format!("GUILD_MEMBER_ADD parse: {e}"))
            }

            "GUILD_MEMBER_REMOVE" => {
                let guild_id = d.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                d.get("user")
                    .cloned()
                    .ok_or_else(|| "no user field".to_string())
                    .and_then(|u| serde_json::from_value::<DiscordUser>(u)
                        .map_err(|e| format!("GUILD_MEMBER_REMOVE parse: {e}")))
                    .map(|user| DiscordEvent::GuildMemberRemove { guild_id, user })
            }

            _ => {
                tracing::trace!("Unhandled event: {event}");
                return;
            }
        };

        match result {
            Ok(event) => {
                let _ = self.event_tx.send(event);
            }
            Err(e) => {
                tracing::warn!("Failed to parse {event}: {e}");
            }
        }
    }
}

enum ConnectionResult {
    Reconnect {
        new_seq: Option<u64>,
        new_session_id: Option<String>,
        new_resume_url: Option<String>,
    },
    InvalidSession,
    Fatal(String),
}
