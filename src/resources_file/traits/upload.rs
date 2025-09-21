use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// 文件上传配置
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// 是否覆盖已存在的文件
    pub overwrite: bool,
    /// 分片上传的块大小（字节）
    pub chunk_size: Option<u64>,
    /// 是否启用断点续传
    pub resume: bool,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            overwrite: false,
            chunk_size: Some(5 * 1024 * 1024), // 5MB
            resume: true,
        }
    }
}

/// 上传进度信息
#[derive(Debug, Clone)]
pub struct UploadProgress {
    /// 已上传字节数
    pub uploaded_bytes: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 上传速度（字节/秒）
    pub speed: Option<u64>,
    /// 是否完成
    pub completed: bool,
}

impl UploadProgress {
    /// 计算上传进度百分比
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.uploaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

/// 文件上传 trait
#[async_trait]
pub trait Upload {
    /// 上传文件到 WebDAV 服务器
    /// 
    /// # 参数
    /// * `local_file_path` - 本地文件路径
    /// * `remote_path` - 远程文件路径（相对于服务器根目录）
    /// * `config` - 上传配置
    /// 
    /// # 返回值
    /// 成功时返回上传后的资源文件对象
    async fn upload_file<P: AsRef<Path> + Send>(
        &self,
        local_file_path: P,
        remote_path: &str,
        config: Option<UploadConfig>,
    ) -> Result<Arc<Self>, String>;

    /// 上传字节数据到 WebDAV 服务器
    /// 
    /// # 参数
    /// * `data` - 要上传的字节数据
    /// * `remote_path` - 远程文件路径
    /// * `config` - 上传配置
    async fn upload_bytes(
        &self,
        data: Vec<u8>,
        remote_path: &str,
        config: Option<UploadConfig>,
    ) -> Result<Arc<Self>, String>;

    /// 创建文件夹
    /// 
    /// # 参数
    /// * `remote_path` - 远程文件夹路径
    async fn create_folder(
        &self,
        remote_path: &str,
    ) -> Result<Arc<Self>, String>;
}

/// 文件操作 trait（删除、移动、复制）
#[async_trait]
pub trait FileOperations {
    /// 删除文件或文件夹
    async fn delete(&self) -> Result<(), String>;

    /// 移动文件或文件夹到新位置
    /// 
    /// # 参数
    /// * `destination_path` - 目标路径
    async fn move_to(&self, destination_path: &str) -> Result<Arc<Self>, String>;

    /// 复制文件或文件夹到新位置
    /// 
    /// # 参数
    /// * `destination_path` - 目标路径
    async fn copy_to(&self, destination_path: &str) -> Result<Arc<Self>, String>;

    /// 重命名文件或文件夹
    /// 
    /// # 参数
    /// * `new_name` - 新名称
    async fn rename(&self, new_name: &str) -> Result<Arc<Self>, String>;
}

/// 批量操作 trait
#[async_trait]
pub trait BatchOperations {
    /// 批量上传文件
    /// 
    /// # 参数
    /// * `files` - 本地文件路径和远程路径的映射
    /// * `config` - 上传配置
    async fn batch_upload(
        &self,
        files: Vec<(String, String)>, // (local_path, remote_path)
        config: Option<UploadConfig>,
    ) -> Result<Vec<Result<Arc<Self>, String>>, String>;

    /// 批量删除文件
    /// 
    /// # 参数
    /// * `paths` - 要删除的远程路径列表
    async fn batch_delete(
        &self,
        paths: Vec<String>,
    ) -> Result<Vec<Result<(), String>>, String>;
}
