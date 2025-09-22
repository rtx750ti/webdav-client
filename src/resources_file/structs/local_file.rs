use tokio::fs::File;
use tokio_util::io::ReaderStream;
use super::upload_conflict::{UploadResult, UploadConflict, ExistingFileInfo};
use crate::client::structs::client_key::ClientKey;

/// 本地文件的两种形态
#[derive(Debug)]
pub enum LocalFileEnum {
    /// 小文件，直接持有 File
    File(File),
    
    /// 大文件，流式读取（抽丝）
    StreamFile(ReaderStream<File>),
}

/// 上传分片黑名单，这些路径模式将禁用分片上传
pub const CHUNKED_UPLOAD_BLACKLIST: [&str; 3] = [
    ".tmp",      // 临时文件
    ".log",      // 日志文件
    ".config",   // 配置文件
];

/// 检查是否在分片上传黑名单中
pub fn is_chunked_upload_blacklisted(target_path: &str) -> bool {
    CHUNKED_UPLOAD_BLACKLIST
        .iter()
        .any(|pattern| target_path.contains(pattern))
}

/// 本地文件上传对象
#[derive(Debug)]
pub struct LocalFile {
    /// 文件数据源
    pub(crate) source: LocalFileEnum,
    /// 目标路径
    pub(crate) target_path: String,
    /// 客户端密钥
    pub(crate) client_key: ClientKey,
    /// 是否启用分片上传（None=使用全局配置, Some(true/false)=局部覆盖）
    pub(crate) enable_chunked: Option<bool>,
}

impl LocalFile {
    /// 创建本地文件上传对象
    /// 
    /// # 参数
    /// * `source` - 文件数据源
    /// * `target_path` - 远程目标路径
    /// * `key` - 客户端密钥
    pub fn new(
        source: LocalFileEnum,
        target_path: String,
        key: &ClientKey,
    ) -> Self {
        Self {
            source,
            target_path,
            client_key: key.clone(),
            enable_chunked: None, // 默认使用全局配置
        }
    }
    
    /// 手动禁用分片上传（局部配置优先级高于全局配置）
    pub fn disable_chunked(mut self) -> Self {
        self.enable_chunked = Some(false);
        self
    }
    
    /// 手动启用分片上传（局部配置优先级高于全局配置）
    pub fn enable_chunked(mut self) -> Self {
        self.enable_chunked = Some(true);
        self
    }
    
    /// 获取目标路径
    pub fn target_path(&self) -> &str {
        &self.target_path
    }

    /// 获取分片配置
    pub fn get_enable_chunked(&self) -> Option<bool> {
        self.enable_chunked
    }

    /// 获取客户端密钥
    pub fn client_key(&self) -> &ClientKey {
        &self.client_key
    }
    
    /// 判断是否应该使用分片上传
    ///
    /// 优先级（从高到低）：
    /// 1. 黑名单检查 (最高优先级，直接禁用分片)
    /// 2. LocalFile.enable_chunked (局部配置)
    /// 3. GlobalConfig.enable_chunked_upload (全局配置)
    /// 4. 默认值 (true)
    pub fn should_use_chunked(&self, global_config: &crate::global_config::GlobalConfig) -> bool {
        // 1. 检查黑名单
        if is_chunked_upload_blacklisted(&self.target_path) {
            return false;
        }

        // 2. 局部配置优先级最高
        if let Some(local_chunked) = self.enable_chunked {
            return local_chunked;
        }

        // 3. 使用全局配置
        #[cfg(feature = "reactive")]
        {
            global_config
                .get_current()
                .map(|cfg| cfg.enable_chunked_upload)
                .unwrap_or(true)
        }

        #[cfg(not(feature = "reactive"))]
        {
            global_config.enable_chunked_upload
        }
    }

    /// 检测上传冲突（在实际上传前进行检查）
    ///
    /// # 返回值
    /// * `Ok(None)` - 无冲突，可以直接上传
    /// * `Ok(Some(conflict))` - 发现冲突，需要用户决策
    /// * `Err(error)` - 检测过程中发生错误
    pub async fn detect_upload_conflict(&self) -> Result<Option<UploadConflict>, String> {
        use reqwest::Method;
        use crate::resources_file::traits_impl::impl_upload::local_file_upload::utils::{
            create_http_client_from_key, build_upload_url_from_key
        };

        // 创建HTTP客户端
        let http_client = create_http_client_from_key(&self.client_key)?;
        let target_url = build_upload_url_from_key(&self.client_key, &self.target_path);

        // 使用HEAD请求检查文件是否存在
        let response = http_client
            .request(Method::HEAD, &target_url)
            .send()
            .await
            .map_err(|e| format!("检查文件存在性失败: {}", e))?;

        match response.status().as_u16() {
            200 => {
                // 文件存在，返回冲突
                Ok(Some(UploadConflict::AlreadyExists))
            }
            404 => {
                // 文件不存在，无冲突
                Ok(None)
            }
            403 => {
                // 权限不足
                Ok(Some(UploadConflict::PermissionDenied))
            }
            _ => {
                // 其他状态码，可能是服务器错误
                Err(format!("检查文件状态时服务器返回: {}", response.status()))
            }
        }
    }

    /// 获取现有文件信息（如果文件存在）
    pub async fn get_existing_file_info(&self) -> Result<Option<ExistingFileInfo>, String> {
        use reqwest::Method;
        use crate::resources_file::traits_impl::impl_upload::local_file_upload::utils::{
            create_http_client_from_key, build_upload_url_from_key
        };

        let http_client = create_http_client_from_key(&self.client_key)?;
        let target_url = build_upload_url_from_key(&self.client_key, &self.target_path);

        let response = http_client
            .request(Method::HEAD, &target_url)
            .send()
            .await
            .map_err(|e| format!("获取文件信息失败: {}", e))?;

        if response.status() == 200 {
            let size = response
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            let last_modified = response
                .headers()
                .get("last-modified")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| chrono::DateTime::parse_from_rfc2822(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            Ok(Some(ExistingFileInfo {
                size,
                last_modified,
                etag,
            }))
        } else {
            Ok(None)
        }
    }

    /// 上传方法（默认带冲突检测）
    ///
    /// # 返回值
    /// * `UploadResult` - 上传结果，可能是成功、冲突或错误
    pub async fn upload(self) -> UploadResult {
        let target_path = self.target_path.clone();

        // 首先检测冲突
        match self.detect_upload_conflict().await {
            Ok(Some(conflict)) => {
                // 发现冲突，获取现有文件信息
                let existing_file_info = self.get_existing_file_info().await.unwrap_or(None);

                UploadResult::Conflict {
                    conflict_info: super::upload_conflict::ConflictInfo {
                        conflict_type: conflict,
                        target_path,
                        existing_file_info,
                    },
                }
            }
            Ok(None) => {
                // 无冲突，直接上传
                self.upload_without_conflict_check().await
            }
            Err(error) => {
                // 检测过程中发生错误
                UploadResult::Error {
                    target_path,
                    error_message: error,
                }
            }
        }
    }

    /// 使用冲突解决策略上传
    ///
    /// # 参数
    /// * `resolution` - 冲突解决策略
    pub async fn upload_with_resolution(mut self, resolution: super::upload_conflict::ConflictResolution) -> UploadResult {
        use super::upload_conflict::ConflictResolution;

        match resolution {
            ConflictResolution::Overwrite => {
                // 直接覆盖，跳过冲突检测
                self.force_upload().await
            }
            ConflictResolution::Rename(new_name) => {
                // 重命名后上传
                self.target_path = new_name;
                self.upload().await
            }
            ConflictResolution::Skip => {
                // 跳过上传
                UploadResult::Success {
                    target_path: self.target_path,
                    file_size: 0,
                    upload_time: std::time::Duration::from_secs(0),
                }
            }
            ConflictResolution::Abort => {
                // 中止上传
                UploadResult::Error {
                    target_path: self.target_path,
                    error_message: "用户中止上传".to_string(),
                }
            }
        }
    }

    /// 强制上传（跳过冲突检测，直接覆盖）
    /// 仅在明确需要覆盖时使用
    pub async fn force_upload(self) -> UploadResult {
        use crate::resources_file::traits_impl::impl_upload::local_file_upload::LocalFileUpload;

        let target_path = self.target_path.clone();
        let start_time = std::time::Instant::now();

        match LocalFileUpload::upload_internal(self).await {
            Ok(_) => {
                let upload_time = start_time.elapsed();
                // 这里需要获取文件大小，暂时设为0
                UploadResult::Success {
                    target_path,
                    file_size: 0, // TODO: 从LocalFile获取实际文件大小
                    upload_time,
                }
            }
            Err(error) => UploadResult::Error {
                target_path,
                error_message: error,
            },
        }
    }

    /// 内部使用：不进行冲突检测的上传
    async fn upload_without_conflict_check(self) -> UploadResult {
        use crate::resources_file::traits_impl::impl_upload::local_file_upload::LocalFileUpload;

        let target_path = self.target_path.clone();
        let start_time = std::time::Instant::now();

        match LocalFileUpload::upload_internal(self).await {
            Ok(_) => {
                let upload_time = start_time.elapsed();
                // 这里需要获取文件大小，暂时设为0
                UploadResult::Success {
                    target_path,
                    file_size: 0, // TODO: 从LocalFile获取实际文件大小
                    upload_time,
                }
            }
            Err(error) => UploadResult::Error {
                target_path,
                error_message: error,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_chunked_upload_blacklist() {
        assert!(is_chunked_upload_blacklisted("/path/to/file.tmp"));
        assert!(is_chunked_upload_blacklisted("/logs/app.log"));
        assert!(is_chunked_upload_blacklisted("/etc/app.config"));
        assert!(!is_chunked_upload_blacklisted("/data/document.pdf"));
    }
    
    #[tokio::test]
    async fn test_local_file_creation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();

        let file = File::open(temp_file.path()).await.unwrap();
        let key = ClientKey::new("http://example.com", "user").unwrap();

        let local_file = LocalFile::new(
            LocalFileEnum::File(file),
            "/remote/test.txt".to_string(),
            &key,
        );

        assert_eq!(local_file.target_path(), "/remote/test.txt");
        assert_eq!(local_file.client_key(), &key);
    }
    
    #[tokio::test]
    async fn test_chunked_configuration() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();

        let file = File::open(temp_file.path()).await.unwrap();
        let key = ClientKey::new("http://example.com", "user").unwrap();

        // 测试禁用分片
        let local_file = LocalFile::new(
            LocalFileEnum::File(file),
            "/remote/test.txt".to_string(),
            &key,
        ).disable_chunked();

        let global_config = crate::global_config::GlobalConfig::default();
        assert!(!local_file.should_use_chunked(&global_config));
    }
}
