//! Storage API 处理器

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::models::{ApiResponse, DeleteRequest, FileListQuery};
use crate::upload::StorageManager;

/// API 状态
#[derive(Clone)]
pub struct StorageState {
    pub storage_manager: StorageManager,
}

/// 注册路由
pub fn routes(state: StorageState) -> Router {
    Router::new()
        .route("/api/storage/upload", post(handle_upload))
        .route("/api/storage/files/:owner_id/:file_id", get(handle_download))
        .route("/api/storage/files/:owner_id/metadata", get(handle_metadata))
        .route("/api/storage/files", get(handle_list))
        .route("/api/storage/files", delete(handle_delete))
        .with_state(state)
}

/// 上传文件处理
async fn handle_upload(
    State(_state): State<StorageState>,
    mut form: Multipart,
) -> impl IntoResponse {
    while let Some(field) = form.next_field().await.ok().flatten() {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap_or_default();

        match name.as_str() {
            "folder" => {
                tracing::info!("Upload folder: {}", String::from_utf8_lossy(&data));
            }
            "file" => {
                tracing::info!("Upload file size: {}", data.len());
            }
            _ => {}
        }
    }

    ApiResponse::success(serde_json::json!({
        "message": "Upload endpoint received",
    }))
}

/// 下载文件处理
async fn handle_download(
    State(state): State<StorageState>,
    Path((owner_id, file_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let owner_id = match Uuid::parse_str(&owner_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid owner_id").into_response(),
    };

    let file_id = match Uuid::parse_str(&file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };

    let path = state.storage_manager.get_storage_path(&owner_id, &file_id, false);

    if !path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let bytes = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to read file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Read failed").into_response();
        }
    };

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id))],
        bytes,
    ).into_response()
}

/// 获取文件元数据
async fn handle_metadata(
    State(state): State<StorageState>,
    Path((owner_id, file_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let owner_id = match Uuid::parse_str(&owner_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid owner_id").into_response(),
    };

    let file_id = match Uuid::parse_str(&file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };

    let path = state.storage_manager.get_storage_path(&owner_id, &file_id, false);

    if !path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let metadata = match tokio::fs::metadata(&path).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to get metadata: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Metadata failed").into_response();
        }
    };

    Json(serde_json::json!({
        "file_id": file_id,
        "owner_id": owner_id,
        "size": metadata.len(),
    }))
    .into_response()
}

/// 文件列表
async fn handle_list(
    State(_state): State<StorageState>,
    Query(query): Query<FileListQuery>,
) -> impl IntoResponse {
    ApiResponse::success(serde_json::json!({
        "files": Vec::<serde_json::Value>::new(),
        "total": 0,
        "limit": query.limit,
        "offset": query.offset,
    }))
}

/// 删除文件
async fn handle_delete(
    State(state): State<StorageState>,
    Json(req): Json<DeleteRequest>,
) -> impl IntoResponse {
    let mut deleted_count = 0u32;

    for file_id_str in &req.file_ids {
        if let Ok(file_id) = Uuid::parse_str(file_id_str) {
            let mut owner_dirs = match tokio::fs::read_dir(&state.storage_manager.config.storage_root.join("files")).await {
                Ok(d) => d,
                Err(_) => continue,
            };

            while let Some(entry) = owner_dirs.next_entry().await.transpose() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        tracing::error!("Failed to read directory entry: {}", e);
                        continue;
                    }
                };
                let file_path = entry.path().join(format!("{}.dat", file_id));
                if file_path.exists() {
                    if let Err(e) = tokio::fs::remove_file(&file_path).await {
                        tracing::error!("Failed to delete file: {}", e);
                    } else {
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    ApiResponse::success(serde_json::json!({
        "deleted_count": deleted_count,
        "total_requested": req.file_ids.len(),
    }))
}