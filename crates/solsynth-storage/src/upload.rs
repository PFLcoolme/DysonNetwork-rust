//! 文件上传处理模块

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::{FileType, StorageQuota};

/// 存储配置
#[derive(Clone)]
pub struct StorageConfig {
    pub storage_root: PathBuf,
    pub max_file_size: u64,
    pub chunk_size: usize,
    pub enable_encryption: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./storage"),
            max_file_size: 500 * 1024 * 1024, // 500 MB
            chunk_size: 8 * 1024 * 1024,      // 8 MB
            enable_encryption: false,
        }
    }
}

/// 文件存储管理器
#[derive(Clone)]
pub struct StorageManager {
    pub config: StorageConfig,
}

impl StorageManager {
    pub fn new(config: StorageConfig) -> Self {
        // 确保存储目录存在
        fs::create_dir_all(&config.storage_root).ok();
        Self { config }
    }

    /// 初始化存储目录结构
    pub fn init_storage_dirs(&self) -> Result<()> {
        let dirs = ["images", "videos", "audio", "documents", "archives", "tmp"];
        for dir in &dirs {
            fs::create_dir_all(self.config.storage_root.join(dir))?;
        }
        Ok(())
    }

    /// 获取文件存储路径
    pub fn get_storage_path(&self, owner_id: &Uuid, file_id: &Uuid, is_tmp: bool) -> PathBuf {
        let prefix = if is_tmp { "tmp" } else { "files" };
        let user_dir = self.config.storage_root.join(prefix).join(owner_id.to_string());
        user_dir.join(format!("{}.dat", file_id))
    }

    /// 计算文件 checksum
    pub async fn calculate_checksum(path: &PathBuf) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        Ok(hex::encode(hasher.finalize()))
    }

    /// 获取文件类型
    pub fn get_file_type(&self, mime_type: &str) -> FileType {
        if mime_type.starts_with("image/") {
            FileType::Image
        } else if mime_type.starts_with("video/") {
            FileType::Video
        } else if mime_type.starts_with("audio/") {
            FileType::Audio
        } else if mime_type.starts_with("application/pdf") || mime_type.starts_with("text/") {
            FileType::Document
        } else if mime_type.contains("zip") || mime_type.contains("rar") || mime_type.contains("tar") {
            FileType::Archive
        } else {
            FileType::Other
        }
    }

    /// 检查存储配额
    pub fn check_quota(&self, quota: &StorageQuota, file_size: u64) -> bool {
        quota.has_space(file_size)
    }

    /// 存储临时文件
    pub async fn store_temp_file(
        &self,
        file_id: &Uuid,
        data: &[u8],
    ) -> Result<PathBuf> {
        let path = self.get_storage_path(&Uuid::nil(), file_id, true);
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        fs::write(&path, data)?;
        Ok(path)
    }

    /// 完成文件移动 (从临时目录到最终目录)
    pub fn finalize_file(
        &self,
        temp_path: &PathBuf,
        final_path: &PathBuf,
    ) -> Result<()> {
        let dir = final_path.parent().unwrap();
        fs::create_dir_all(dir)?;
        fs::rename(temp_path, final_path)?;
        Ok(())
    }

    /// 获取文件大小
    pub fn get_file_size(path: &PathBuf) -> Result<u64> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// 清理临时文件
    pub fn cleanup_temp(&self, file_id: &Uuid) -> Result<()> {
        let path = self.get_storage_path(&Uuid::nil(), file_id, true);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// 构建文件存储路径
pub fn build_storage_path(
    root: &PathBuf,
    owner_id: &Uuid,
    file_id: &Uuid,
    folder: Option<&str>,
) -> PathBuf {
    let base = root.join("files").join(owner_id.to_string());
    
    if let Some(folder) = folder {
        base.join(folder).join(format!("{}.dat", file_id))
    } else {
        base.join(format!("{}.dat", file_id))
    }
}

/// 构建文件访问 URL
pub fn build_file_url(base_url: &str, owner_id: &Uuid, file_id: &Uuid) -> String {
    format!("{}/api/storage/files/{}/{}", base_url, owner_id, file_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checksum_calculation() {
        let temp_dir = std::env::temp_dir().join("solsynth_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test data").unwrap();
        
        let checksum = StorageManager::calculate_checksum(&test_file).await.unwrap();
        assert_eq!(checksum.len(), 64); // SHA-256
        
        // 清理
        fs::remove_file(&test_file).ok();
        fs::remove_dir_all(&temp_dir).ok();
    }
}