use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits::upload::UploadConfig;
use reqwest::Client;
use std::path::PathBuf;
use crate::resources_file::traits_impl::impl_upload::simple_upload::{simple_upload, SimpleUploadArgs};
use crate::resources_file::traits_impl::impl_upload::chunked_upload::{chunked_upload, ChunkedUploadArgs};

pub struct HandleUploadArgs {
    pub(crate) local_file_path: PathBuf,
    pub(crate) upload_url: String,
    pub(crate) http_client: Client,
    pub(crate) config: UploadConfig,
    pub(crate) global_config: GlobalConfig,
    #[cfg(feature = "reactive")]
    pub(crate) inner_state: ReactiveFileProperty,
    #[cfg(feature = "reactive")]
    pub(crate) inner_config: ReactiveConfig,
}

/// 获取大文件阈值
fn get_large_file_threshold(global_config: &GlobalConfig) -> Result<u64, String> {
    #[cfg(feature = "reactive")]
    {
        global_config
            .get_current()
            .map(|config| config.large_file_threshold)
            .ok_or_else(|| "无法获取全局配置".to_string())
    }
    
    #[cfg(not(feature = "reactive"))]
    {
        Ok(global_config.large_file_threshold)
    }
}

/// 统一处理非分片上传
async fn upload_without_chunking(args: HandleUploadArgs) -> Result<(), String> {
    let simple_upload_args = SimpleUploadArgs {
        local_file_path: args.local_file_path,
        upload_url: args.upload_url,
        http_client: args.http_client,
        config: args.config,
        global_config: args.global_config,
        #[cfg(feature = "reactive")]
        inner_state: args.inner_state,
        #[cfg(feature = "reactive")]
        inner_config: args.inner_config,
    };
    
    simple_upload(simple_upload_args)
        .await
        .map_err(|e| format!("[simple_upload] {}", e))
}

/// 统一处理分片上传
async fn upload_with_chunking(args: HandleUploadArgs) -> Result<(), String> {
    let chunked_upload_args = ChunkedUploadArgs {
        local_file_path: args.local_file_path,
        upload_url: args.upload_url,
        http_client: args.http_client,
        config: args.config,
        global_config: args.global_config,
        #[cfg(feature = "reactive")]
        inner_state: args.inner_state,
        #[cfg(feature = "reactive")]
        inner_config: args.inner_config,
    };
    
    chunked_upload(chunked_upload_args)
        .await
        .map_err(|e| format!("[chunked_upload] {}", e))
}

pub async fn handle_upload(args: HandleUploadArgs) -> Result<(), String> {
    // 检查本地文件是否存在
    if !args.local_file_path.exists() {
        return Err(format!(
            "本地文件不存在: {}",
            args.local_file_path.display()
        ));
    }
    
    // 检查是否为文件（不是目录）
    if !args.local_file_path.is_file() {
        return Err(format!(
            "路径不是文件: {}",
            args.local_file_path.display()
        ));
    }
    
    // 获取文件大小
    let file_size = std::fs::metadata(&args.local_file_path)
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    // 检查是否需要分片上传
    let threshold = get_large_file_threshold(&args.global_config)?;
    let use_chunked = if let Some(chunk_size) = args.config.chunk_size {
        file_size > threshold && chunk_size > 0
    } else {
        false
    };
    
    if use_chunked {
        upload_with_chunking(args).await
    } else {
        upload_without_chunking(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_get_large_file_threshold() {
        let global_config = GlobalConfig::default();
        let threshold = get_large_file_threshold(&global_config).unwrap();
        assert!(threshold > 0);
    }
    
    #[tokio::test]
    async fn test_handle_upload_file_not_exists() {
        let args = HandleUploadArgs {
            local_file_path: PathBuf::from("/non/existent/file.txt"),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig::default(),
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        let result = handle_upload(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("本地文件不存在"));
    }
    
    #[tokio::test]
    async fn test_handle_upload_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        
        let args = HandleUploadArgs {
            local_file_path: temp_dir.path().to_path_buf(),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig::default(),
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        let result = handle_upload(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("路径不是文件"));
    }
    
    #[tokio::test]
    async fn test_handle_upload_small_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        
        let args = HandleUploadArgs {
            local_file_path: temp_file.path().to_path_buf(),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig {
                chunk_size: None, // 禁用分片上传
                ..Default::default()
            },
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        // 这个测试会失败，因为没有真实的服务器，但至少验证了文件检查逻辑
        let result = handle_upload(args).await;
        // 应该是网络错误，而不是文件检查错误
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.contains("本地文件不存在"));
        assert!(!error.contains("路径不是文件"));
    }
}
