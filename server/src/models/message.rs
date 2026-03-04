use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::attachment::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Message with author info joined
#[derive(Debug, Serialize, FromRow)]
pub struct MessageWithAuthor {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub author_username: String,
    pub author_display_name: Option<String>,
    pub author_avatar_url: Option<String>,
}

/// Full message response including attachments
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub author_username: String,
    pub author_display_name: Option<String>,
    pub author_avatar_url: Option<String>,
    pub attachments: Vec<Attachment>,
}

impl MessageResponse {
    pub fn from_msg(msg: MessageWithAuthor, attachments: Vec<Attachment>) -> Self {
        Self {
            id: msg.id,
            channel_id: msg.channel_id,
            author_id: msg.author_id,
            content: msg.content,
            edited_at: msg.edited_at,
            created_at: msg.created_at,
            author_username: msg.author_username,
            author_display_name: msg.author_display_name,
            author_avatar_url: msg.author_avatar_url,
            attachments,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: Option<String>,
    pub attachment_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub limit: Option<i64>,
}
