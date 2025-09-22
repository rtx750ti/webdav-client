use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits::upload::UploadConfig;
use reqwest::Client;
use std::path::PathBuf;

// 重用分片上传模块的HTTP功能
use crate::resources_file::traits_impl::impl_upload::chunked_upload::{
    send_simple_upload_request,
    check_remote_file_exists,
    infer_content_type,
    validate_response,
};

/// 简单上传参数结构体（保持向后兼容）
pub struct SimpleUploadArgs {
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

/// 执行简单上传（重构后的实现）
/// 
/// # 参数
/// * `args` - 简单上传参数
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn simple_upload(args: SimpleUploadArgs) -> Result<(), String> {
    println!("📤 开始简单上传: {}", args.local_file_path.display());
    
    // 检查文件是否存在
    if !args.local_file_path.exists() {
        return Err(format!(
            "本地文件不存在: {}",
            args.local_file_path.display()
        ));
    }
    
    // 检查是否为文件
    if !args.local_file_path.is_file() {
        return Err(format!(
            "路径不是文件: {}",
            args.local_file_path.display()
        ));
    }
    
    // 检查是否需要覆盖文件
    if !args.config.overwrite {
        let exists = check_remote_file_exists(&args.http_client, &args.upload_url).await?;
        if exists {
            return Err(format!(
                "远程文件已存在，且未启用覆盖选项: {}",
                args.upload_url
            ));
        }
    }
    
    // 获取文件大小
    let file_size = std::fs::metadata(&args.local_file_path)
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    println!("📊 文件信息: 大小 {} 字节", file_size);
    
    // 读取整个文件
    let file_data = tokio::fs::read(&args.local_file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 推断内容类型
    let content_type = infer_content_type(&args.local_file_path.to_string_lossy());
    println!("📋 内容类型: {}", content_type);
    
    // 发送简单上传请求
    let response = send_simple_upload_request(
        &args.http_client,
        &args.upload_url,
        file_data,
        Some(&content_type),
    ).await?;
    
    // 验证响应
    validate_response(&response, "简单上传")?;
    
    println!("✅ 简单上传成功: {}", args.local_file_path.display());
    Ok(())
}

/// 从内存数据执行简单上传
/// 
/// # 参数
/// * `data` - 要上传的数据
/// * `upload_url` - 上传URL
/// * `http_client` - HTTP客户端
/// * `content_type` - 内容类型（可选）
/// * `overwrite` - 是否覆盖已存在的文件
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn simple_upload_bytes(
    data: Vec<u8>,
    upload_url: &str,
    http_client: &Client,
    content_type: Option<&str>,
    overwrite: bool,
) -> Result<(), String> {
    println!("📤 开始字节数据上传: {} 字节", data.len());
    
    // 检查是否需要覆盖文件
    if !overwrite {
        let exists = check_remote_file_exists(http_client, upload_url).await?;
        if exists {
            return Err(format!(
                "远程文件已存在，且未启用覆盖选项: {}",
                upload_url
            ));
        }
    }
    
    // 发送简单上传请求
    let response = send_simple_upload_request(
        http_client,
        upload_url,
        data,
        content_type,
    ).await?;
    
    // 验证响应
    validate_response(&response, "字节数据上传")?;
    
    println!("✅ 字节数据上传成功");
    Ok(())
}

/// 检查文件是否适合简单上传
/// 
/// # 参数
/// * `file_path` - 文件路径
/// * `max_simple_upload_size` - 简单上传的最大文件大小
/// 
/// # 返回值
/// * `Result<bool, String>` - 是否适合简单上传
pub async fn is_suitable_for_simple_upload(
    file_path: &PathBuf,
    max_simple_upload_size: u64,
) -> Result<bool, String> {
    // 获取文件大小
    let file_size = std::fs::metadata(file_path)
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    // 检查文件大小
    if file_size > max_simple_upload_size {
        return Ok(false);
    }
    
    // 检查是否在黑名单中（虽然简单上传通常不受黑名单限制）
    // 这里可以添加其他适用性检查
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_simple_upload_file_not_found() {
        let args = SimpleUploadArgs {
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
        
        let result = simple_upload(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("本地文件不存在"));
    }

    #[tokio::test]
    async fn test_is_suitable_for_simple_upload() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Small test file";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        let file_path = temp_file.path().to_path_buf();
        
        // 测试小文件适合简单上传
        let result = is_suitable_for_simple_upload(&file_path, 1024).await.unwrap();
        assert!(result);
        
        // 测试大文件不适合简单上传
        let result = is_suitable_for_simple_upload(&file_path, 10).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_simple_upload_bytes() {
        let test_data = b"Test data for upload".to_vec();
        
        // 这个测试会失败，因为我们没有真实的服务器
        // 但它会测试参数验证逻辑
        let result = simple_upload_bytes(
            test_data,
            "http://example.com/upload",
            &Client::new(),
            Some("text/plain"),
            true,
        ).await;
        
        // 预期会因为网络错误失败
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(!error_msg.contains("远程文件已存在"));
    }
}
