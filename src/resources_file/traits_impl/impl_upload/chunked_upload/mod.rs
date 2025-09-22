mod file;
mod http;
mod task;
pub mod black_list;

use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits::upload::UploadConfig;
use reqwest::Client;
use std::path::PathBuf;

// 重新导出黑名单功能
pub use black_list::{
    is_chunked_upload_blacklisted,
    should_use_chunked_upload,
    CHUNKED_UPLOAD_BLACKLIST,
};

// 重新导出文件操作功能
use file::{
    get_file_size,
    open_file_for_read,
    calculate_chunk_count,
};

// 重新导出HTTP功能
pub use http::{
    send_simple_upload_request,
    check_remote_file_exists,
    infer_content_type,
    validate_response,
};

// 重新导出任务管理功能
use task::{
    create_chunk_tasks,
    execute_all_chunk_tasks,
    build_upload_task_args,
    calculate_optimal_concurrency,
};

/// 分片上传参数结构体（保持向后兼容）
pub struct ChunkedUploadArgs {
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

/// 执行分片上传（重构后的实现）
/// 
/// # 参数
/// * `args` - 分片上传参数
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn chunked_upload(args: ChunkedUploadArgs) -> Result<(), String> {
    println!("🚀 开始分片上传: {}", args.local_file_path.display());
    
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
    let total_size = get_file_size(&args.local_file_path).await?;
    
    // 获取分片大小
    let chunk_size = args.config.chunk_size.unwrap_or(5 * 1024 * 1024); // 默认5MB
    
    // 检查是否真的需要分片上传
    let file_path_str = args.local_file_path.to_string_lossy();
    if !should_use_chunked_upload(&file_path_str, total_size, chunk_size) {
        println!("📝 文件不适合分片上传，使用简单上传");
        return simple_upload_fallback(args).await;
    }
    
    // 计算分片数量
    let chunk_count = calculate_chunk_count(total_size, chunk_size);
    println!(
        "📊 分片上传信息: 文件大小 {} 字节, 分片大小 {} 字节, 分片数量 {}",
        total_size, chunk_size, chunk_count
    );
    
    // 打开文件
    let mut file = open_file_for_read(&args.local_file_path).await?;
    
    // 创建分片任务
    let tasks = create_chunk_tasks(&mut file, total_size, chunk_size).await?;
    
    // 计算最优并发数
    let max_concurrent = calculate_optimal_concurrency(chunk_count, 3);
    
    // 构建任务参数
    let task_args = build_upload_task_args(
        args.http_client,
        args.upload_url,
        &file_path_str,
        total_size,
        chunk_size,
        max_concurrent,
    );
    
    // 执行所有分片上传任务
    execute_all_chunk_tasks(tasks, task_args).await?;
    
    println!("🎉 分片上传完成: {}", args.local_file_path.display());
    Ok(())
}

/// 简单上传回退方案
/// 
/// # 参数
/// * `args` - 分片上传参数
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
async fn simple_upload_fallback(args: ChunkedUploadArgs) -> Result<(), String> {
    // 读取整个文件
    let file_data = tokio::fs::read(&args.local_file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 推断内容类型
    let content_type = infer_content_type(&args.local_file_path.to_string_lossy());
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_chunked_upload_file_not_found() {
        let args = ChunkedUploadArgs {
            local_file_path: PathBuf::from("/non/existent/file.txt"),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig {
                chunk_size: Some(1024),
                ..Default::default()
            },
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        let result = chunked_upload(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("本地文件不存在"));
    }

    #[tokio::test]
    async fn test_chunked_upload_blacklist_fallback() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::with_suffix(".tmp").unwrap();
        let test_data = b"This is a test file that should use simple upload due to blacklist";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        let args = ChunkedUploadArgs {
            local_file_path: temp_file.path().to_path_buf(),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig {
                chunk_size: Some(10), // 很小的分片大小，但由于黑名单应该回退到简单上传
                overwrite: true,
                ..Default::default()
            },
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        // 这个测试会失败，因为我们没有真实的服务器
        // 但它会测试黑名单逻辑和简单上传回退
        let result = chunked_upload(args).await;
        // 预期会因为网络错误失败，但不是因为文件不存在
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(!error_msg.contains("本地文件不存在"));
    }
}
