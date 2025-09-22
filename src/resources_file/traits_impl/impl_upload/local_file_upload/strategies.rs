use crate::global_config::GlobalConfig;
use crate::public::enums::methods::WebDavMethod;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, CONTENT_LENGTH};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;
use std::sync::Arc;
use tokio::sync::Semaphore;
use futures_util::StreamExt;

use super::utils::{build_upload_url_from_key, get_chunk_size};

/// 简单上传实现
///
/// # 参数
/// * `file` - 文件句柄
/// * `target_path` - 目标路径
/// * `client_key` - 客户端密钥
/// * `global_config` - 全局配置
/// * `http_client` - HTTP客户端
///
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn simple_upload_from_file(
    mut file: File,
    target_path: &str,
    client_key: &crate::client::structs::client_key::ClientKey,
    _global_config: &GlobalConfig,
    http_client: &Client,
) -> Result<(), String> {
    println!("📤 开始简单上传到: {}", target_path);

    // 构建上传URL
    let upload_url = build_upload_url_from_key(client_key, target_path);
    
    // 获取文件大小
    let file_size = file.metadata()
        .await
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    println!("📊 文件大小: {} 字节", file_size);
    
    // 读取整个文件
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 构建PUT请求
    let method = WebDavMethod::PUT
        .to_head_method()
        .map_err(|e| format!("构建PUT方法失败: {}", e))?;
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&file_size.to_string())
        .map_err(|e| format!("设置Content-Length失败: {}", e))?);
    
    // 发送请求
    let response = http_client
        .request(method, &upload_url)
        .headers(headers)
        .body(buffer)
        .send()
        .await
        .map_err(|e| format!("发送PUT请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "简单上传失败: {} - {}",
            response.status(),
            upload_url
        ));
    }
    
    println!("✅ 简单上传成功: {}", target_path);
    Ok(())
}

/// 分片上传实现
///
/// # 参数
/// * `file` - 文件句柄
/// * `target_path` - 目标路径
/// * `client_key` - 客户端密钥
/// * `global_config` - 全局配置
/// * `http_client` - HTTP客户端
///
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn chunked_upload_from_file(
    mut file: File,
    target_path: &str,
    client_key: &crate::client::structs::client_key::ClientKey,
    global_config: &GlobalConfig,
    http_client: &Client,
) -> Result<(), String> {
    println!("🚀 开始分片上传到: {}", target_path);

    // 构建上传URL
    let upload_url = build_upload_url_from_key(client_key, target_path);
    
    // 获取文件大小和分片大小
    let file_size = file.metadata()
        .await
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    let chunk_size = get_chunk_size(global_config);
    let total_chunks = ((file_size + chunk_size - 1) / chunk_size) as usize;
    
    println!(
        "📊 分片信息: 文件大小 {} 字节, 分片大小 {} 字节, 总分片数 {}",
        file_size, chunk_size, total_chunks
    );
    
    // 创建信号量控制并发
    let max_concurrent = 3;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    // 创建上传任务
    let mut tasks = Vec::new();
    
    for chunk_index in 0..total_chunks {
        let start = chunk_index as u64 * chunk_size;
        let end = std::cmp::min(start + chunk_size, file_size);
        let current_chunk_size = (end - start) as usize;
        
        // 读取分片数据
        file.seek(SeekFrom::Start(start))
            .await
            .map_err(|e| format!("文件定位失败: {}", e))?;
        
        let mut chunk_data = vec![0u8; current_chunk_size];
        file.read_exact(&mut chunk_data)
            .await
            .map_err(|e| format!("读取文件分片失败: {}", e))?;
        
        // 克隆必要的数据
        let http_client = http_client.clone();
        let upload_url = upload_url.clone();
        let semaphore = Arc::clone(&semaphore);
        
        // 创建异步任务
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            upload_chunk(
                &http_client,
                &upload_url,
                chunk_data,
                start,
                end,
                file_size,
                chunk_index,
            ).await
        });
        
        tasks.push(task);
    }
    
    // 等待所有分片上传完成
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (index, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(Ok(())) => {
                success_count += 1;
                println!("✅ 分片 {} 上传成功", index);
            }
            Ok(Err(e)) => {
                error_count += 1;
                eprintln!("❌ 分片 {} 上传失败: {}", index, e);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("❌ 分片 {} 任务失败: {}", index, e);
            }
        }
    }
    
    if error_count > 0 {
        return Err(format!(
            "分片上传失败: {} 个成功, {} 个失败",
            success_count, error_count
        ));
    }
    
    println!("🎉 分片上传完全成功: {}", target_path);
    Ok(())
}

/// 流式上传实现
///
/// # 参数
/// * `stream` - 文件流
/// * `target_path` - 目标路径
/// * `client_key` - 客户端密钥
/// * `global_config` - 全局配置
/// * `http_client` - HTTP客户端
///
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub async fn stream_upload(
    mut stream: ReaderStream<File>,
    target_path: &str,
    client_key: &crate::client::structs::client_key::ClientKey,
    global_config: &GlobalConfig,
    http_client: &Client,
) -> Result<(), String> {
    println!("🌊 开始流式上传到: {}", target_path);

    // 构建上传URL
    let upload_url = build_upload_url_from_key(client_key, target_path);
    
    // 收集流数据并分片上传
    let mut chunks = Vec::new();
    let chunk_size = get_chunk_size(global_config);
    let mut current_chunk = Vec::new();
    
    while let Some(bytes_result) = stream.next().await {
        let bytes = bytes_result.map_err(|e| format!("读取流数据失败: {}", e))?;
        current_chunk.extend_from_slice(&bytes);
        
        if current_chunk.len() >= chunk_size as usize {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
    }
    
    // 处理最后一个分片
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    println!("📊 流式上传信息: 总分片数 {}", chunks.len());
    
    // 上传所有分片
    for (index, chunk_data) in chunks.into_iter().enumerate() {
        upload_chunk(
            &http_client,
            &upload_url,
            chunk_data,
            (index as u64) * chunk_size,
            ((index + 1) as u64) * chunk_size,
            0, // 流式上传时总大小未知
            index,
        ).await.map_err(|e| format!("流式上传分片 {} 失败: {}", index, e))?;
        
        println!("✅ 流式分片 {} 上传成功", index);
    }
    
    println!("🎉 流式上传完全成功: {}", target_path);
    Ok(())
}

/// 上传单个分片
/// 
/// # 参数
/// * `http_client` - HTTP客户端
/// * `upload_url` - 上传URL
/// * `chunk_data` - 分片数据
/// * `start` - 起始位置
/// * `end` - 结束位置
/// * `total_size` - 总大小
/// * `chunk_index` - 分片索引
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
async fn upload_chunk(
    http_client: &Client,
    upload_url: &str,
    chunk_data: Vec<u8>,
    start: u64,
    end: u64,
    total_size: u64,
    chunk_index: usize,
) -> Result<(), String> {
    let method = WebDavMethod::PUT
        .to_head_method()
        .map_err(|e| format!("构建PUT方法失败: {}", e))?;
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&chunk_data.len().to_string())
        .map_err(|e| format!("设置Content-Length失败: {}", e))?);
    
    // 如果知道总大小，设置Content-Range头
    if total_size > 0 {
        let range_header = format!("bytes {}-{}/{}", start, end - 1, total_size);
        headers.insert("Content-Range", HeaderValue::from_str(&range_header)
            .map_err(|e| format!("设置Content-Range失败: {}", e))?);
    }
    
    let response = http_client
        .request(method, upload_url)
        .headers(headers)
        .body(chunk_data)
        .send()
        .await
        .map_err(|e| format!("发送分片请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "分片 {} 上传失败: {} - {}",
            chunk_index,
            response.status(),
            upload_url
        ));
    }
    
    Ok(())
}
