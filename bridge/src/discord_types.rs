use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Discord Gateway opcodes
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

/// Raw gateway payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPayload {
    pub op: Opcode,
    #[serde(default)]
    pub d: Option<serde_json::Value>,
    #[serde(default)]
    pub s: Option<u64>,
    #[serde(default)]
    pub t: Option<String>,
}

// ─── Discord data types ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub discriminator: Option<String>,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
}

impl DiscordUser {
    pub fn display_name(&self) -> &str {
        self.global_name
            .as_deref()
            .unwrap_or(&self.username)
    }

    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|hash| {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{hash}.{ext}?size=256", self.id)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGuild {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub channels: Option<Vec<DiscordChannel>>,
    #[serde(default)]
    pub members: Option<Vec<DiscordGuildMember>>,
    #[serde(default)]
    pub roles: Option<Vec<DiscordRole>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordChannel {
    pub id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub recipients: Option<Vec<DiscordUser>>,
}

impl DiscordChannel {
    /// 0 = text, 2 = voice, 4 = category, 5 = announcement, 13 = stage, 15 = forum
    pub fn is_text(&self) -> bool {
        matches!(self.channel_type, 0 | 5)
    }

    pub fn is_dm(&self) -> bool {
        matches!(self.channel_type, 1 | 3) // DM or group DM
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGuildMember {
    #[serde(default)]
    pub user: Option<DiscordUser>,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordRole {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub position: i32,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub author: DiscordUser,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub attachments: Vec<DiscordAttachment>,
    #[serde(default)]
    pub embeds: Vec<serde_json::Value>,
    #[serde(default)]
    pub reactions: Option<Vec<DiscordReaction>>,
    #[serde(rename = "type")]
    pub message_type: u8,
    #[serde(default)]
    pub referenced_message: Option<Box<DiscordMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordAttachment {
    pub id: String,
    pub filename: String,
    pub url: String,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordReaction {
    pub count: u32,
    pub me: bool,
    pub emoji: DiscordEmoji,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmoji {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

impl DiscordEmoji {
    pub fn display(&self) -> String {
        if let Some(id) = &self.id {
            format!("<:{}:{}>", self.name.as_deref().unwrap_or("emoji"), id)
        } else {
            self.name.clone().unwrap_or_default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordReady {
    pub user: DiscordUser,
    pub guilds: Vec<serde_json::Value>, // Unavailable guilds at first
    pub session_id: String,
    pub resume_gateway_url: String,
    #[serde(default)]
    pub private_channels: Vec<DiscordChannel>,
}

/// Typing start event
#[derive(Debug, Clone, Deserialize)]
pub struct TypingStart {
    pub channel_id: String,
    pub user_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// Message delete event
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelete {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// Message reaction add/remove
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReactionUpdate {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub emoji: DiscordEmoji,
}

/// Guild create (full guild object sent after READY)
pub type GuildCreate = DiscordGuild;

// ─── Gateway intents ───

pub const INTENT_GUILDS: u64 = 1 << 0;
pub const INTENT_GUILD_MEMBERS: u64 = 1 << 1;
pub const INTENT_GUILD_MESSAGES: u64 = 1 << 9;
pub const INTENT_GUILD_MESSAGE_REACTIONS: u64 = 1 << 10;
pub const INTENT_GUILD_MESSAGE_TYPING: u64 = 1 << 11;
pub const INTENT_DIRECT_MESSAGES: u64 = 1 << 12;
pub const INTENT_DIRECT_MESSAGE_REACTIONS: u64 = 1 << 13;
pub const INTENT_DIRECT_MESSAGE_TYPING: u64 = 1 << 14;
pub const INTENT_MESSAGE_CONTENT: u64 = 1 << 15;

/// All intents needed for full bridge
pub const BRIDGE_INTENTS: u64 = INTENT_GUILDS
    | INTENT_GUILD_MEMBERS
    | INTENT_GUILD_MESSAGES
    | INTENT_GUILD_MESSAGE_REACTIONS
    | INTENT_GUILD_MESSAGE_TYPING
    | INTENT_DIRECT_MESSAGES
    | INTENT_DIRECT_MESSAGE_REACTIONS
    | INTENT_DIRECT_MESSAGE_TYPING
    | INTENT_MESSAGE_CONTENT;
