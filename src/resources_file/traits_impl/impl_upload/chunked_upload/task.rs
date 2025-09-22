use std::sync::Arc;
use tokio::sync::Semaphore;
use reqwest::Client;
use crate::resources_file::traits_impl::impl_upload::chunked_upload::http::{
    send_chunked_upload_request, validate_response, infer_content_type
};
use crate::resources_file::traits_impl::impl_upload::chunked_upload::file::{
    calculate_chunk_range, read_file_chunk
};
use tokio::fs::File;

/// 上传任务参数
#[derive(Clone)]
pub struct UploadTaskArgs {
    /// HTTP客户端
    pub client: Client,
    /// 上传URL
    pub upload_url: String,
    /// 文件总大小
    pub total_size: u64,
    /// 分片大小
    pub chunk_size: u64,
    /// 内容类型
    pub content_type: String,
    /// 并发控制信号量
    pub semaphore: Arc<Semaphore>,
}

/// 单个分片上传任务
pub struct ChunkUploadTask {
    /// 分片索引
    pub chunk_index: usize,
    /// 分片数据
    pub chunk_data: Vec<u8>,
    /// 起始位置
    pub start: u64,
    /// 结束位置
    pub end: u64,
}

/// 创建分片上传任务列表
/// 
/// # 参数
/// * `file` - 文件句柄
/// * `total_size` - 文件总大小
/// * `chunk_size` - 分片大小
/// 
/// # 返回值
/// * `Result<Vec<ChunkUploadTask>, String>` - 任务列表
pub async fn create_chunk_tasks(
    file: &mut File,
    total_size: u64,
    chunk_size: u64,
) -> Result<Vec<ChunkUploadTask>, String> {
    let chunk_count = ((total_size + chunk_size - 1) / chunk_size) as usize;
    let mut tasks = Vec::with_capacity(chunk_count);
    
    for chunk_index in 0..chunk_count {
        let (start, size) = calculate_chunk_range(chunk_index, chunk_size, total_size);
        let end = start + size;
        
        // 读取分片数据
        let chunk_data = read_file_chunk(file, start, size as usize).await?;
        
        tasks.push(ChunkUploadTask {
            chunk_index,
            chunk_data,
            start,
            end,
        });
    }
    
    Ok(tasks)
}

/// 执行单个分片上传任务
/// 
/// # 参数
/// * `task` - 上传任务
/// * `args` - 任务参数
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn execute_chunk_upload_task(
    task: ChunkUploadTask,
    args: &UploadTaskArgs,
) -> Result<(), String> {
    // 获取信号量许可
    let _permit = args.semaphore
        .acquire()
        .await
        .map_err(|e| format!("获取信号量许可失败: {}", e))?;
    
    // 发送分片上传请求
    let response = send_chunked_upload_request(
        &args.client,
        &args.upload_url,
        task.chunk_data,
        task.start,
        task.end,
        args.total_size,
        Some(&args.content_type),
    ).await?;
    
    // 验证响应
    validate_response(&response, &format!("分片 {} 上传", task.chunk_index))?;
    
    println!("✅ 分片 {} 上传成功 ({}-{})", task.chunk_index, task.start, task.end - 1);
    Ok(())
}

/// 并发执行所有分片上传任务
/// 
/// # 参数
/// * `tasks` - 任务列表
/// * `args` - 任务参数
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn execute_all_chunk_tasks(
    tasks: Vec<ChunkUploadTask>,
    args: UploadTaskArgs,
) -> Result<(), String> {
    let total_chunks = tasks.len();
    println!("🚀 开始并发上传 {} 个分片", total_chunks);
    
    // 创建异步任务
    let mut handles = Vec::new();
    
    for task in tasks {
        let args_clone = args.clone();
        let handle = tokio::spawn(async move {
            execute_chunk_upload_task(task, &args_clone).await
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();
    
    for (index, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(Ok(())) => {
                success_count += 1;
            }
            Ok(Err(e)) => {
                error_count += 1;
                errors.push(format!("分片 {} 失败: {}", index, e));
                eprintln!("❌ 分片 {} 上传失败: {}", index, e);
            }
            Err(e) => {
                error_count += 1;
                errors.push(format!("分片 {} 任务失败: {}", index, e));
                eprintln!("❌ 分片 {} 任务失败: {}", index, e);
            }
        }
    }
    
    // 检查结果
    if error_count > 0 {
        return Err(format!(
            "分片上传失败: {} 个成功, {} 个失败\n错误详情:\n{}",
            success_count,
            error_count,
            errors.join("\n")
        ));
    }
    
    println!("🎉 所有分片上传成功: {} 个分片", success_count);
    Ok(())
}

/// 构建上传任务参数
/// 
/// # 参数
/// * `client` - HTTP客户端
/// * `upload_url` - 上传URL
/// * `file_path` - 文件路径（用于推断内容类型）
/// * `total_size` - 文件总大小
/// * `chunk_size` - 分片大小
/// * `max_concurrent` - 最大并发数
/// 
/// # 返回值
/// * `UploadTaskArgs` - 任务参数
pub fn build_upload_task_args(
    client: Client,
    upload_url: String,
    file_path: &str,
    total_size: u64,
    chunk_size: u64,
    max_concurrent: usize,
) -> UploadTaskArgs {
    UploadTaskArgs {
        client,
        upload_url,
        total_size,
        chunk_size,
        content_type: infer_content_type(file_path),
        semaphore: Arc::new(Semaphore::new(max_concurrent)),
    }
}

/// 计算合适的并发数
/// 
/// # 参数
/// * `chunk_count` - 分片数量
/// * `max_concurrent` - 最大并发数限制
/// 
/// # 返回值
/// * 建议的并发数
pub fn calculate_optimal_concurrency(chunk_count: usize, max_concurrent: usize) -> usize {
    if chunk_count <= 1 {
        1
    } else if chunk_count <= 5 {
        std::cmp::min(chunk_count, 2)
    } else if chunk_count <= 20 {
        std::cmp::min(chunk_count, 3)
    } else {
        std::cmp::min(chunk_count, max_concurrent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_optimal_concurrency() {
        assert_eq!(calculate_optimal_concurrency(1, 10), 1);
        assert_eq!(calculate_optimal_concurrency(3, 10), 2);
        assert_eq!(calculate_optimal_concurrency(10, 10), 3);
        assert_eq!(calculate_optimal_concurrency(50, 10), 10);
        assert_eq!(calculate_optimal_concurrency(50, 5), 5);
    }

    #[test]
    fn test_build_upload_task_args() {
        let client = Client::new();
        let args = build_upload_task_args(
            client,
            "http://example.com/upload".to_string(),
            "/path/to/file.jpg",
            1024 * 1024,
            64 * 1024,
            3,
        );
        
        assert_eq!(args.upload_url, "http://example.com/upload");
        assert_eq!(args.total_size, 1024 * 1024);
        assert_eq!(args.chunk_size, 64 * 1024);
        assert_eq!(args.content_type, "image/jpeg");
    }
}
