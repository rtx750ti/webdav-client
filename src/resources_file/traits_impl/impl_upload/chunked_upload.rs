use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits::upload::UploadConfig;
use crate::public::enums::methods::WebDavMethod;
use reqwest::{Client, Body};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, CONTENT_LENGTH, CONTENT_RANGE};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use std::sync::Arc;
use tokio::sync::Semaphore;

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

/// 上传单个分片
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
    
    // 设置 Content-Type
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    
    // 设置 Content-Length
    headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&chunk_data.len().to_string())
            .map_err(|e| format!("设置Content-Length失败: {}", e))?,
    );
    
    // 设置 Content-Range (对于分片上传)
    let range_header = format!("bytes {}-{}/{}", start, end - 1, total_size);
    headers.insert(
        CONTENT_RANGE,
        HeaderValue::from_str(&range_header)
            .map_err(|e| format!("设置Content-Range失败: {}", e))?,
    );
    
    // 构建分片上传的 URL（可能需要添加分片标识）
    let chunk_url = format!("{}?chunk={}", upload_url, chunk_index);
    
    let response = http_client
        .request(method, &chunk_url)
        .headers(headers)
        .body(Body::from(chunk_data))
        .send()
        .await
        .map_err(|e| format!("发送分片上传请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "分片上传失败: {} - 分片 {} ({})",
            response.status(),
            chunk_index,
            chunk_url
        ));
    }
    
    println!("✅ 分片 {} 上传成功: bytes {}-{}", chunk_index, start, end - 1);
    Ok(())
}

/// 完成分片上传（合并分片）
async fn finalize_chunked_upload(
    http_client: &Client,
    upload_url: &str,
    total_chunks: usize,
    total_size: u64,
) -> Result<(), String> {
    // 发送完成请求，告诉服务器合并所有分片
    let finalize_url = format!("{}?finalize=true&chunks={}&size={}", upload_url, total_chunks, total_size);
    
    let response = http_client
        .post(&finalize_url)
        .send()
        .await
        .map_err(|e| format!("发送完成请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "完成分片上传失败: {} - {}",
            response.status(),
            finalize_url
        ));
    }
    
    println!("✅ 分片上传完成，已合并 {} 个分片", total_chunks);
    Ok(())
}

pub async fn chunked_upload(args: ChunkedUploadArgs) -> Result<(), String> {
    // 获取文件大小
    let file_size = std::fs::metadata(&args.local_file_path)
        .map_err(|e| format!("获取文件大小失败: {}", e))?
        .len();
    
    // 获取分片大小
    let chunk_size = args.config.chunk_size.unwrap_or(5 * 1024 * 1024); // 默认 5MB
    
    if chunk_size == 0 {
        return Err("分片大小不能为0".to_string());
    }
    
    // 计算分片数量
    let total_chunks = ((file_size + chunk_size - 1) / chunk_size) as usize;
    
    println!(
        "🚀 开始分片上传: 文件大小 {} 字节, 分片大小 {} 字节, 总分片数 {}",
        file_size, chunk_size, total_chunks
    );
    
    #[cfg(feature = "reactive")]
    {
        // 更新上传状态
        let _ = args.inner_state.set_upload_total_bytes(file_size);
        let _ = args.inner_state.set_upload_bytes(0);
    }
    
    // 打开文件
    let mut file = File::open(&args.local_file_path)
        .await
        .map_err(|e| format!("打开文件失败: {}", e))?;
    
    // 创建信号量来控制并发数
    let max_concurrent = 3; // 最大并发上传数
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
        let http_client = args.http_client.clone();
        let upload_url = args.upload_url.clone();
        let semaphore = Arc::clone(&semaphore);
        
        #[cfg(feature = "reactive")]
        let inner_state = args.inner_state.clone();
        
        // 创建异步任务
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let result = upload_chunk(
                &http_client,
                &upload_url,
                chunk_data,
                start,
                end,
                file_size,
                chunk_index,
            ).await;
            
            #[cfg(feature = "reactive")]
            if result.is_ok() {
                let _ = inner_state.add_upload_bytes(end - start);
            }
            
            result
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
                println!("分片 {} 上传成功", index);
            }
            Ok(Err(e)) => {
                error_count += 1;
                eprintln!("分片 {} 上传失败: {}", index, e);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("分片 {} 任务失败: {}", index, e);
            }
        }
    }
    
    if error_count > 0 {
        return Err(format!(
            "分片上传失败: {} 个成功, {} 个失败",
            success_count, error_count
        ));
    }
    
    // 完成分片上传（合并分片）
    finalize_chunked_upload(&args.http_client, &args.upload_url, total_chunks, file_size).await?;
    
    println!(
        "🎉 分片上传完全成功: {} -> {}",
        args.local_file_path.display(),
        args.upload_url
    );
    
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
        assert!(result.unwrap_err().contains("获取文件大小失败"));
    }
    
    #[tokio::test]
    async fn test_chunked_upload_zero_chunk_size() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        
        let args = ChunkedUploadArgs {
            local_file_path: temp_file.path().to_path_buf(),
            upload_url: "http://example.com/upload".to_string(),
            http_client: Client::new(),
            config: UploadConfig {
                chunk_size: Some(0), // 无效的分片大小
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
        assert!(result.unwrap_err().contains("分片大小不能为0"));
    }
}
