use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, CONTENT_LENGTH};
use crate::public::enums::methods::WebDavMethod;

/// 构建上传请求的HTTP头
/// 
/// # 参数
/// * `content_length` - 内容长度
/// * `content_type` - 内容类型（可选）
/// 
/// # 返回值
/// * `Result<HeaderMap, String>` - HTTP头映射
pub fn build_upload_headers(
    content_length: u64,
    content_type: Option<&str>,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    
    // 设置内容类型
    let ct = content_type.unwrap_or("application/octet-stream");
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(ct)
            .map_err(|e| format!("设置Content-Type失败: {}", e))?
    );
    
    // 设置内容长度
    headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&content_length.to_string())
            .map_err(|e| format!("设置Content-Length失败: {}", e))?
    );
    
    Ok(headers)
}

/// 构建分片上传请求的HTTP头
/// 
/// # 参数
/// * `chunk_data_len` - 分片数据长度
/// * `start` - 分片起始位置
/// * `end` - 分片结束位置（不包含）
/// * `total_size` - 文件总大小
/// * `content_type` - 内容类型（可选）
/// 
/// # 返回值
/// * `Result<HeaderMap, String>` - HTTP头映射
pub fn build_chunked_upload_headers(
    chunk_data_len: usize,
    start: u64,
    end: u64,
    total_size: u64,
    content_type: Option<&str>,
) -> Result<HeaderMap, String> {
    let mut headers = build_upload_headers(chunk_data_len as u64, content_type)?;
    
    // 设置Content-Range头（用于分片上传）
    let range_header = format!("bytes {}-{}/{}", start, end - 1, total_size);
    headers.insert(
        "Content-Range",
        HeaderValue::from_str(&range_header)
            .map_err(|e| format!("设置Content-Range失败: {}", e))?
    );
    
    Ok(headers)
}

/// 发送简单上传请求
/// 
/// # 参数
/// * `client` - HTTP客户端
/// * `upload_url` - 上传URL
/// * `data` - 要上传的数据
/// * `content_type` - 内容类型（可选）
/// 
/// # 返回值
/// * `Result<Response, String>` - HTTP响应
pub async fn send_simple_upload_request(
    client: &Client,
    upload_url: &str,
    data: Vec<u8>,
    content_type: Option<&str>,
) -> Result<Response, String> {
    let method = WebDavMethod::PUT
        .to_head_method()
        .map_err(|e| format!("构建PUT方法失败: {}", e))?;
    
    let headers = build_upload_headers(data.len() as u64, content_type)?;
    
    let response = client
        .request(method, upload_url)
        .headers(headers)
        .body(data)
        .send()
        .await
        .map_err(|e| format!("发送PUT请求失败: {}", e))?;
    
    Ok(response)
}

/// 发送分片上传请求
/// 
/// # 参数
/// * `client` - HTTP客户端
/// * `upload_url` - 上传URL
/// * `chunk_data` - 分片数据
/// * `start` - 分片起始位置
/// * `end` - 分片结束位置（不包含）
/// * `total_size` - 文件总大小
/// * `content_type` - 内容类型（可选）
/// 
/// # 返回值
/// * `Result<Response, String>` - HTTP响应
pub async fn send_chunked_upload_request(
    client: &Client,
    upload_url: &str,
    chunk_data: Vec<u8>,
    start: u64,
    end: u64,
    total_size: u64,
    content_type: Option<&str>,
) -> Result<Response, String> {
    let method = WebDavMethod::PUT
        .to_head_method()
        .map_err(|e| format!("构建PUT方法失败: {}", e))?;
    
    let headers = build_chunked_upload_headers(
        chunk_data.len(),
        start,
        end,
        total_size,
        content_type,
    )?;
    
    let response = client
        .request(method, upload_url)
        .headers(headers)
        .body(chunk_data)
        .send()
        .await
        .map_err(|e| format!("发送分片PUT请求失败: {}", e))?;
    
    Ok(response)
}

/// 检查远程文件是否存在
/// 
/// # 参数
/// * `client` - HTTP客户端
/// * `url` - 文件URL
/// 
/// # 返回值
/// * `Result<bool, String>` - 文件是否存在
pub async fn check_remote_file_exists(
    client: &Client,
    url: &str,
) -> Result<bool, String> {
    let response = client
        .head(url)
        .send()
        .await
        .map_err(|e| format!("检查远程文件失败: {}", e))?;
    
    Ok(response.status().is_success())
}

/// 根据文件扩展名推断内容类型
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * 内容类型字符串
pub fn infer_content_type(file_path: &str) -> String {
    if let Some(extension) = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        match extension.to_lowercase().as_str() {
            "txt" => "text/plain".to_string(),
            "html" | "htm" => "text/html".to_string(),
            "css" => "text/css".to_string(),
            "js" => "application/javascript".to_string(),
            "json" => "application/json".to_string(),
            "xml" => "application/xml".to_string(),
            "pdf" => "application/pdf".to_string(),
            "zip" => "application/zip".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "gif" => "image/gif".to_string(),
            "mp4" => "video/mp4".to_string(),
            "mp3" => "audio/mpeg".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    } else {
        "application/octet-stream".to_string()
    }
}

/// 验证HTTP响应是否成功
/// 
/// # 参数
/// * `response` - HTTP响应
/// * `operation` - 操作描述（用于错误信息）
/// 
/// # 返回值
/// * `Result<(), String>` - 成功或错误信息
pub fn validate_response(response: &Response, operation: &str) -> Result<(), String> {
    if !response.status().is_success() {
        return Err(format!(
            "{} 失败: HTTP {} - {}",
            operation,
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_upload_headers() {
        let headers = build_upload_headers(1024, Some("text/plain")).unwrap();
        
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            "text/plain"
        );
        assert_eq!(
            headers.get(CONTENT_LENGTH).unwrap(),
            "1024"
        );
    }

    #[test]
    fn test_build_chunked_upload_headers() {
        let headers = build_chunked_upload_headers(
            512,    // chunk_data_len
            0,      // start
            512,    // end
            1024,   // total_size
            Some("application/octet-stream")
        ).unwrap();
        
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
        assert_eq!(
            headers.get(CONTENT_LENGTH).unwrap(),
            "512"
        );
        assert_eq!(
            headers.get("Content-Range").unwrap(),
            "bytes 0-511/1024"
        );
    }

    #[test]
    fn test_infer_content_type() {
        assert_eq!(infer_content_type("test.txt"), "text/plain");
        assert_eq!(infer_content_type("index.html"), "text/html");
        assert_eq!(infer_content_type("data.json"), "application/json");
        assert_eq!(infer_content_type("image.jpg"), "image/jpeg");
        assert_eq!(infer_content_type("video.mp4"), "video/mp4");
        assert_eq!(infer_content_type("unknown.xyz"), "application/octet-stream");
        assert_eq!(infer_content_type("no_extension"), "application/octet-stream");
    }
}
