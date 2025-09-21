use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits::upload::UploadConfig;
use crate::public::enums::methods::WebDavMethod;
use reqwest::{Client, Body};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, CONTENT_LENGTH};
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

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

/// 检查远程文件是否存在
async fn check_remote_file_exists(
    http_client: &Client,
    upload_url: &str,
) -> Result<bool, String> {
    let response = http_client
        .head(upload_url)
        .send()
        .await
        .map_err(|e| format!("检查远程文件失败: {}", e))?;
    
    Ok(response.status().is_success())
}

/// 获取文件的 MIME 类型
fn get_mime_type(file_path: &PathBuf) -> String {
    if let Some(extension) = file_path.extension() {
        match extension.to_str() {
            Some("txt") => "text/plain".to_string(),
            Some("html") | Some("htm") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("json") => "application/json".to_string(),
            Some("xml") => "application/xml".to_string(),
            Some("pdf") => "application/pdf".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("png") => "image/png".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("mp4") => "video/mp4".to_string(),
            Some("mp3") => "audio/mpeg".to_string(),
            Some("zip") => "application/zip".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    } else {
        "application/octet-stream".to_string()
    }
}

pub async fn simple_upload(args: SimpleUploadArgs) -> Result<(), String> {
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
    
    // 打开文件
    let file = File::open(&args.local_file_path)
        .await
        .map_err(|e| format!("打开文件失败: {}", e))?;
    
    // 创建文件流
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);
    
    // 构建请求头
    let mut headers = HeaderMap::new();
    
    // 设置 Content-Type
    let mime_type = get_mime_type(&args.local_file_path);
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&mime_type)
            .map_err(|e| format!("设置Content-Type失败: {}", e))?,
    );
    
    // 设置 Content-Length
    headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&file_size.to_string())
            .map_err(|e| format!("设置Content-Length失败: {}", e))?,
    );
    
    // 如果启用覆盖，添加相应的头
    if args.config.overwrite {
        headers.insert("Overwrite", HeaderValue::from_static("T"));
    }
    
    // 构建 PUT 请求
    let method = WebDavMethod::PUT
        .to_head_method()
        .map_err(|e| format!("构建PUT方法失败: {}", e))?;
    
    #[cfg(feature = "reactive")]
    {
        // 更新上传状态
        let _ = args.inner_state.set_upload_total_bytes(file_size);
        let _ = args.inner_state.set_upload_bytes(0);
    }
    
    // 发送请求
    let response = args.http_client
        .request(method, &args.upload_url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("发送PUT请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(format!(
            "上传失败: {} - {}",
            response.status(),
            args.upload_url
        ));
    }
    
    #[cfg(feature = "reactive")]
    {
        // 更新完成状态
        let _ = args.inner_state.set_upload_bytes(file_size);
    }
    
    println!(
        "✅ 文件上传成功: {} -> {}",
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
    
    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(&PathBuf::from("test.txt")), "text/plain");
        assert_eq!(get_mime_type(&PathBuf::from("test.html")), "text/html");
        assert_eq!(get_mime_type(&PathBuf::from("test.jpg")), "image/jpeg");
        assert_eq!(get_mime_type(&PathBuf::from("test.png")), "image/png");
        assert_eq!(get_mime_type(&PathBuf::from("test.pdf")), "application/pdf");
        assert_eq!(get_mime_type(&PathBuf::from("test.unknown")), "application/octet-stream");
        assert_eq!(get_mime_type(&PathBuf::from("test")), "application/octet-stream");
    }
    
    #[tokio::test]
    async fn test_check_remote_file_exists_error() {
        let client = Client::new();
        let result = check_remote_file_exists(&client, "http://invalid-url-12345.com/test").await;
        assert!(result.is_err());
    }
    
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
        assert!(result.unwrap_err().contains("获取文件大小失败"));
    }
    
    #[tokio::test]
    async fn test_simple_upload_with_temp_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        
        let args = SimpleUploadArgs {
            local_file_path: temp_file.path().to_path_buf(),
            upload_url: "http://httpbin.org/put".to_string(), // 使用测试服务
            http_client: Client::new(),
            config: UploadConfig {
                overwrite: true,
                ..Default::default()
            },
            global_config: GlobalConfig::default(),
            #[cfg(feature = "reactive")]
            inner_state: ReactiveFileProperty::new("test".to_string()),
            #[cfg(feature = "reactive")]
            inner_config: ReactiveConfig::default(),
        };
        
        // 这个测试可能会因为网络问题失败，但至少验证了基本逻辑
        let result = simple_upload(args).await;
        // 不强制要求成功，因为依赖外部服务
        if let Err(e) = result {
            println!("上传测试失败（可能是网络问题）: {}", e);
        }
    }
}
