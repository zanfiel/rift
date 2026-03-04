use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::config::Config;
use crate::error::AppError;

/// Metadata returned from upload — client includes these IDs when sending a message
#[derive(Debug, Serialize)]
pub struct UploadedFile {
    pub upload_id: Uuid,
    pub filename: String,
    pub url: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
}

/// In-memory store for pending uploads (files saved to disk, not yet linked to a message)
/// Key: upload_id, Value: file metadata
pub type PendingUploads = std::sync::Arc<dashmap::DashMap<Uuid, PendingUpload>>;

#[derive(Debug, Clone)]
pub struct PendingUpload {
    pub uploader_id: Uuid,
    pub filename: String,
    pub stored_filename: String,
    pub url: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
}

/// POST /api/upload — upload one or more files, returns metadata for each
pub async fn upload_files(
    State(config): State<Config>,
    State(pending): State<PendingUploads>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadedFile>>, AppError> {
    let mut results = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let original_filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed".into());

        let content_type = field.content_type().map(|s| s.to_string());

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        if data.len() > config.max_upload_bytes {
            return Err(AppError::BadRequest(format!(
                "File too large (max {} bytes)",
                config.max_upload_bytes
            )));
        }

        if data.is_empty() {
            continue;
        }

        // Generate unique stored filename to avoid collisions
        let upload_id = Uuid::new_v4();
        let extension = std::path::Path::new(&original_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let stored_filename = if extension.is_empty() {
            upload_id.to_string()
        } else {
            format!("{upload_id}.{extension}")
        };

        let file_path = std::path::Path::new(&config.upload_dir).join(&stored_filename);
        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

        let url = format!("/uploads/{stored_filename}");
        let size_bytes = data.len() as i64;

        // Store pending upload metadata
        pending.insert(
            upload_id,
            PendingUpload {
                uploader_id: auth.user_id,
                filename: original_filename.clone(),
                stored_filename: stored_filename.clone(),
                url: url.clone(),
                content_type: content_type.clone(),
                size_bytes,
            },
        );

        results.push(UploadedFile {
            upload_id,
            filename: original_filename,
            url,
            content_type,
            size_bytes,
        });
    }

    if results.is_empty() {
        return Err(AppError::BadRequest("No files uploaded".into()));
    }

    Ok(Json(results))
}
