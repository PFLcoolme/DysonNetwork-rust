//! 文件下载处理模块

use anyhow::Result;
use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::Response,
    Request,
};
use std::path::PathBuf;
use uuid::Uuid;

use crate::upload::{StorageConfig, StorageManager};

/// 下载文件
pub async fn download_file(
    storage_manager: StorageManager,
    owner_id: Uuid,
    file_id: Uuid,
) -> Result<Response<Body>> {
    let path = storage_manager.get_storage_path(&owner_id, &file_id, false);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }

    let file = tokio::fs::File::open(&path).await?;
    let metadata = tokio::fs::metadata(&path).await?;
    let file_size = metadata.len();

    // 读取文件内容
    let mut reader = tokio::io::BufReader::new(file);
    let mut buffer = Vec::new();
    use tokio::io::AsyncReadExt;
    reader.read_to_end(&mut buffer).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id))
        .header(header::CONTENT_LENGTH, file_size)
        .body(Body::from(buffer))?;

    Ok(response)
}

/// 流式下载文件
pub async fn stream_download_file(
    request: Request,
    storage_manager: StorageManager,
    owner_id: Uuid,
    file_id: Uuid,
) -> Result<Response<Body>> {
    let path = storage_manager.get_storage_path(&owner_id, &file_id, false);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }

    let file = tokio::fs::File::open(&path).await?;
    let stream = tokio_util::io::ReaderStream::new(tokio::io::BufReader::new(file));
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id))
        .body(body)?;

    Ok(response)
}

/// 获取文件元数据
pub async fn get_file_metadata(
    storage_manager: StorageManager,
    owner_id: &Uuid,
    file_id: &Uuid,
) -> Result<axum::Json<serde_json::Value>> {
    let path = storage_manager.get_storage_path(owner_id, file_id, false);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }

    let metadata = tokio::fs::metadata(&path).await?;
    let checksum = StorageManager::calculate_checksum(&path).await?;

    let response = serde_json::json!({
        "file_id": file_id.to_string(),
        "owner_id": owner_id.to_string(),
        "size": metadata.len(),
        "checksum": checksum,
    });

    Ok(axum::Json(response))
}

/// 范围请求支持 (用于视频流等)
pub async fn range_download_file(
    request: Request,
    storage_manager: StorageManager,
    owner_id: Uuid,
    file_id: Uuid,
) -> Result<Response<Body>> {
    let path = storage_manager.get_storage_path(&owner_id, &file_id, false);

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }

    let metadata = tokio::fs::metadata(&path).await?;
    let file_size = metadata.len();

    // 解析 Range 头
    let range_header = request.headers().get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("bytes=0-");

    // 简单处理范围请求
    let (start, end) = parse_range(range_header, file_size);
    let content_length = end - start + 1;

    let file = tokio::fs::File::open(&path).await?;
    let mut reader = tokio::io::BufReader::new(file);
    use tokio::io::AsyncSeekExt;
   

 reader.seek(std::io::SeekFrom::Start(start)).await?;    let mut buffer = vec![0u8; content_length as usize];
    reader.read_exact(&mut buffer).await?;

    let response = Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size))
        .header(header::CONTENT_LENGTH, content_length)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id))
        .body(Body::from(buffer))?;

    Ok(response)
}

fn parse_range(range_header: &str, file_size: u64) -> (u64, u64) {
    // 简单实现: bytes=start-end
    let parts: Vec<&str> = range_header.trim_start_matches("bytes=").split('-').collect();
    let start = parts[0].parse::<u64>().unwrap_or(0);
    let end = if parts.len() > 1 && !parts[1].is_empty() {
        parts[1].parse::<u64>().unwrap_or(file_size - 1)
    } else {
        file_size    - 1
    };
 (start, end.min(file_size - 1))
}