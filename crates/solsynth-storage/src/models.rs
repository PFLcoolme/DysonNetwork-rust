//! 文件存储数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 文件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Image = 0,
    Video = 1,
    Audio = 2,
    Document = 3,
    Archive = 4,
    Other = 99,
}

impl FileType {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Image,
            1 => Self::Video,
            2 => Self::Audio,
            3 => Self::Document,
            4 => Self::Archive,
            99 => Self::Other,
            _ => Self::Other,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Image => 0,
            Self::Video => 1,
            Self::Audio => 2,
            Self::Document => 3,
            Self::Archive => 4,
            Self::Other => 99,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Image => "image/*",
            Self::Video => "video/*",
            Self::Audio => "audio/*",
            Self::Document => "application/pdf",
            Self::Archive => "application/zip",
            Self::Other => "application/octet-stream",
        }
    }
}

/// 文件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Uploading = 0,
    Uploaded = 1,
    Deleted = 2,
}

impl FileStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Uploading,
            1 => Self::Uploaded,
            2 => Self::Deleted,
            _ => Self::Uploading,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Uploading => 0,
            Self::Uploaded => 1,
            Self::Deleted => 2,
        }
    }
}

/// 文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub file_type: FileType,
    pub storage_path: String,
    pub encryption_key_id: Option<String>,
    pub checksum: String,
    pub status: FileStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 上传请求
#[derive(Debug, Clone, Deserialize)]
pub struct UploadRequest {
    pub folder: Option<String>,
    pub encrypt: Option<bool>,
}

/// 分片上传请求
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkUploadRequest {
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub folder: Option<String>,
    pub chunk_index: usize,
    pub total_chunks: usize,
}

/// 分片上传初始化响应
#[derive(Debug, Serialize)]
pub struct ChunkUploadInitResponse {
    pub upload_id: String,
    pub chunk_size: usize,
}

/// 分片上传完成响应
#[derive(Debug, Serialize)]
pub struct ChunkUploadCompleteResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub storage_path: String,
}

/// 上传响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub storage_path: String,
    pub url: Option<String>,
}

/// 下载响应
#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub checksum: String,
}

/// 存储空间配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    pub user_id: Uuid,
    pub total_space: u64,       // 总空间 (字节)
    pub used_space: u64,        // 已用空间 (字节)
    pub file_count: u64,        // 文件数量
    pub max_file_size: u64,     // 最大文件大小
}

impl StorageQuota {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            total_space: 10 * 1024 * 1024 * 1024, // 10 GB
            used_space: 0,
            file_count: 0,
            max_file_size: 500 * 1024 * 1024, // 500 MB
        }
    }

    pub fn has_space(&self, file_size: u64) -> bool {
        file_size <= self.max_file_size
            && (self.used_space + file_size) <= self.total_space
    }

    pub fn usage_percentage(&self) -> f64 {
        if self.total_space == 0 {
            0.0
        } else {
            (self.used_space as f64 / self.total_space as f64) * 100.0
        }
    }
}

/// 文件列表查询参数
#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub file_type: Option<i8>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_limit() -> i32 {
    20
}

fn default_sort_order() -> String {
    "desc".to_string()
}

/// 删除请求
#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub file_ids: Vec<String>,
    pub permanent: Option<bool>, // 是否永久删除
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(msg),
        }
    }
}