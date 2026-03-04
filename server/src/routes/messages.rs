use axum::{
    extract::{Path, Query, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::db;
use crate::error::AppError;
use crate::models::message::{EditMessageRequest, MessageQuery, MessageWithAuthor, SendMessageRequest};
use crate::models::permissions::perms;
use crate::ws::gateway::{Gateway, GatewayEvent};

/// GET /api/channels/:channel_id/messages
pub async fn list_messages(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<MessageWithAuthor>>, AppError> {
    let channel = db::get_channel_by_id(&pool, channel_id)
        .await?
        .ok_or(AppError::NotFound("Channel not found".into()))?;

    require_member(&pool, channel.server_id, auth.user_id).await?;

    let messages = db::get_messages(&pool, channel_id, &query).await?;
    Ok(Json(messages))
}

/// POST /api/channels/:channel_id/messages
pub async fn send_message(
    State(pool): State<PgPool>,
    State(gateway): State<Gateway>,
    auth: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageWithAuthor>, AppError> {
    let channel = db::get_channel_by_id(&pool, channel_id)
        .await?
        .ok_or(AppError::NotFound("Channel not found".into()))?;

    require_permission(&pool, channel.server_id, auth.user_id, perms::SEND_MESSAGES).await?;

    let content = req.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("Message cannot be empty".into()));
    }
    if content.len() > 4000 {
        return Err(AppError::BadRequest("Message too long (max 4000 chars)".into()));
    }

    let msg = db::create_message(&pool, channel_id, auth.user_id, content).await?;

    // Broadcast to channel subscribers
    gateway.broadcast_to_channel(
        channel_id,
        GatewayEvent::MessageCreate {
            id: msg.id,
            channel_id: msg.channel_id,
            author_id: msg.author_id,
            author_username: msg.author_username.clone(),
            author_display_name: msg.author_display_name.clone(),
            author_avatar_url: msg.author_avatar_url.clone(),
            content: msg.content.clone(),
            created_at: msg.created_at.to_rfc3339(),
        },
    );

    Ok(Json(msg))
}

/// PATCH /api/channels/:channel_id/messages/:message_id
pub async fn edit_message(
    State(pool): State<PgPool>,
    State(gateway): State<Gateway>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EditMessageRequest>,
) -> Result<Json<MessageWithAuthor>, AppError> {
    let existing = db::get_message_by_id(&pool, message_id)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    if existing.author_id != auth.user_id {
        return Err(AppError::Forbidden);
    }

    let content = req.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("Message cannot be empty".into()));
    }

    let msg = db::update_message(&pool, message_id, content).await?;

    gateway.broadcast_to_channel(
        channel_id,
        GatewayEvent::MessageUpdate {
            id: msg.id,
            channel_id,
            content: msg.content.clone(),
            edited_at: msg.edited_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        },
    );

    Ok(Json(msg))
}

/// DELETE /api/channels/:channel_id/messages/:message_id
pub async fn delete_message(
    State(pool): State<PgPool>,
    State(gateway): State<Gateway>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let existing = db::get_message_by_id(&pool, message_id)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    // Author can delete own messages, or user with MANAGE_MESSAGES
    if existing.author_id != auth.user_id {
        let channel = db::get_channel_by_id(&pool, channel_id)
            .await?
            .ok_or(AppError::NotFound("Channel not found".into()))?;
        require_permission(&pool, channel.server_id, auth.user_id, perms::MANAGE_MESSAGES).await?;
    }

    db::delete_message(&pool, message_id).await?;

    gateway.broadcast_to_channel(
        channel_id,
        GatewayEvent::MessageDelete {
            id: message_id,
            channel_id,
        },
    );

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── Helpers ───

async fn require_member(pool: &PgPool, server_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    if !db::is_member(pool, server_id, user_id).await? {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn require_permission(
    pool: &PgPool,
    server_id: Uuid,
    user_id: Uuid,
    permission: i64,
) -> Result<(), AppError> {
    require_member(pool, server_id, user_id).await?;
    let user_perms = db::get_member_permissions(pool, server_id, user_id).await?;
    if !perms::has(user_perms, permission) {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
