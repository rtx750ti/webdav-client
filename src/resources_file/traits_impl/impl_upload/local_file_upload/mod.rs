mod strategies;
pub mod utils;

use crate::resources_file::structs::local_file::{LocalFile, LocalFileEnum};
use async_trait::async_trait;

// 重新导出策略和工具函数
pub use strategies::{
    simple_upload_from_file,
    chunked_upload_from_file,
    stream_upload,
};

pub use utils::{
    create_http_client_from_key,
    get_default_global_config,
    build_upload_url_from_key,
    get_chunk_size,
};

/// LocalFile上传trait（内部实现）
#[async_trait]
pub trait LocalFileUpload {
    /// 执行实际上传（不进行冲突检测）
    async fn upload_internal(self) -> Result<(), String>;
}

#[async_trait]
impl LocalFileUpload for LocalFile {
    async fn upload_internal(self) -> Result<(), String> {
        // 直接从ClientKey构建所需组件
        let http_client = create_http_client_from_key(&self.client_key)?;
        let global_config = get_default_global_config();

        // 决定上传策略
        let use_chunked = self.should_use_chunked(&global_config);

        match (self.source, use_chunked) {
            (LocalFileEnum::File(file), false) => {
                // 简单上传
                simple_upload_from_file(file, &self.target_path, &self.client_key, &global_config, &http_client).await
            }
            (LocalFileEnum::File(file), true) => {
                // 文件分片上传
                chunked_upload_from_file(file, &self.target_path, &self.client_key, &global_config, &http_client).await
            }
            (LocalFileEnum::StreamFile(stream), _) => {
                // 流式上传（总是分片）
                stream_upload(stream, &self.target_path, &self.client_key, &global_config, &http_client).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::structs::client_key::ClientKey;
    use tokio::fs::File;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_client_key() -> ClientKey {
        ClientKey::new(
            "http://192.168.5.90:36879/",
            "test_user",
        ).unwrap()
    }

    #[tokio::test]
    async fn test_local_file_upload_trait() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for LocalFile upload";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        // 打开文件
        let file = File::open(temp_file.path()).await.unwrap();
        let client_key = create_test_client_key();
        
        // 创建LocalFile
        let local_file = LocalFile::new(
            LocalFileEnum::File(file),
            "/test_upload.txt".to_string(),
            &client_key,
        );
        
        // 测试上传（会失败，因为没有真实的服务器）
        let result = local_file.upload_internal().await;
        assert!(result.is_err());

        // 验证错误信息包含预期的内容
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("全局客户端管理器未实现"));
    }

    #[tokio::test]
    async fn test_local_file_upload_with_chunked_disabled() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for LocalFile upload with chunked disabled";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        // 打开文件
        let file = File::open(temp_file.path()).await.unwrap();
        let client_key = create_test_client_key();
        
        // 创建LocalFile并禁用分片
        let local_file = LocalFile::new(
            LocalFileEnum::File(file),
            "/test_upload_no_chunk.txt".to_string(),
            &client_key,
        ).disable_chunked();
        
        // 验证分片配置
        assert_eq!(local_file.enable_chunked, Some(false));
        
        // 测试上传（会失败，因为没有真实的服务器）
        let result = local_file.upload_internal().await;
        assert!(result.is_err());

        // 验证错误信息包含预期的内容
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("全局客户端管理器未实现"));
    }

    #[tokio::test]
    async fn test_local_file_upload_with_chunked_enabled() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test data for LocalFile upload with chunked enabled";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        // 打开文件
        let file = File::open(temp_file.path()).await.unwrap();
        let client_key = create_test_client_key();
        
        // 创建LocalFile并启用分片
        let local_file = LocalFile::new(
            LocalFileEnum::File(file),
            "/test_upload_chunked.txt".to_string(),
            &client_key,
        ).enable_chunked();
        
        // 验证分片配置
        assert_eq!(local_file.enable_chunked, Some(true));
        
        // 测试上传（会失败，因为没有真实的服务器）
        let result = local_file.upload_internal().await;
        assert!(result.is_err());

        // 验证错误信息包含预期的内容
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("全局客户端管理器未实现"));
    }
}
