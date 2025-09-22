use crate::client::WebDavClient;
use crate::global_config::GlobalConfig;
use reqwest::Client;

/// 创建HTTP客户端（直接从ClientKey构建）
///
/// # 参数
/// * `key` - 客户端密钥
///
/// # 返回值
/// * `Result<Client, String>` - HTTP客户端
pub fn create_http_client_from_key(_key: &crate::client::structs::client_key::ClientKey) -> Result<Client, String> {
    // 直接创建HTTP客户端，不依赖全局管理器
    Ok(Client::new())
}

/// 获取默认全局配置
///
/// # 返回值
/// * `GlobalConfig` - 默认全局配置
pub fn get_default_global_config() -> GlobalConfig {
    GlobalConfig::default()
}

/// 构建上传URL
///
/// # 参数
/// * `key` - 客户端密钥
/// * `target_path` - 目标路径
///
/// # 返回值
/// * `String` - 完整的上传URL
pub fn build_upload_url_from_key(key: &crate::client::structs::client_key::ClientKey, target_path: &str) -> String {
    let base_url = key.get_base_url();
    let mut url = base_url.to_string();

    // 确保base_url以/结尾
    if !url.ends_with('/') {
        url.push('/');
    }

    // 确保target_path不以/开头（避免双斜杠）
    let clean_path = if target_path.starts_with('/') {
        &target_path[1..]
    } else {
        target_path
    };

    format!("{}{}", url, clean_path)
}

/// 获取分片大小
/// 
/// # 参数
/// * `global_config` - 全局配置
/// 
/// # 返回值
/// * 分片大小（字节）
pub fn get_chunk_size(global_config: &GlobalConfig) -> u64 {
    #[cfg(feature = "reactive")]
    {
        global_config
            .get_current()
            .map(|cfg| cfg.chunk_size)
            .unwrap_or(5 * 1024 * 1024) // 默认5MB
    }
    
    #[cfg(not(feature = "reactive"))]
    {
        global_config.chunk_size
    }
}

/// 获取大文件阈值
/// 
/// # 参数
/// * `global_config` - 全局配置
/// 
/// # 返回值
/// * 大文件阈值（字节）
pub fn get_large_file_threshold(global_config: &GlobalConfig) -> u64 {
    #[cfg(feature = "reactive")]
    {
        global_config
            .get_current()
            .map(|cfg| cfg.large_file_threshold)
            .unwrap_or(10 * 1024 * 1024) // 默认10MB
    }
    
    #[cfg(not(feature = "reactive"))]
    {
        // 假设GlobalConfig有这个字段，如果没有则使用默认值
        10 * 1024 * 1024 // 默认10MB
    }
}

/// 获取最大并发数
///
/// # 参数
/// * `global_config` - 全局配置
///
/// # 返回值
/// * 最大并发数
pub fn get_max_concurrent_uploads(_global_config: &GlobalConfig) -> usize {
    // 暂时使用固定值，后续可以从配置中读取
    3 // 默认3个并发
}

/// 检查是否启用分片上传
/// 
/// # 参数
/// * `global_config` - 全局配置
/// 
/// # 返回值
/// * 是否启用分片上传
pub fn is_chunked_upload_enabled(global_config: &GlobalConfig) -> bool {
    #[cfg(feature = "reactive")]
    {
        global_config
            .get_current()
            .map(|cfg| cfg.enable_chunked_upload)
            .unwrap_or(true) // 默认启用
    }
    
    #[cfg(not(feature = "reactive"))]
    {
        // 假设GlobalConfig有这个字段，如果没有则使用默认值
        true // 默认启用
    }
}

/// 构建完整的上传URL
/// 
/// # 参数
/// * `base_url` - 基础URL
/// * `target_path` - 目标路径
/// 
/// # 返回值
/// * 完整的上传URL
pub fn build_upload_url(base_url: &str, target_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = if target_path.starts_with('/') {
        target_path
    } else {
        &format!("/{}", target_path)
    };
    
    format!("{}{}", base, path)
}

/// 验证目标路径格式
/// 
/// # 参数
/// * `target_path` - 目标路径
/// 
/// # 返回值
/// * `Result<(), String>` - 验证结果
pub fn validate_target_path(target_path: &str) -> Result<(), String> {
    if target_path.is_empty() {
        return Err("目标路径不能为空".to_string());
    }
    
    if target_path.contains("..") {
        return Err("目标路径不能包含相对路径".to_string());
    }
    
    // 检查路径是否包含非法字符
    let illegal_chars = ['<', '>', ':', '"', '|', '?', '*'];
    for ch in illegal_chars.iter() {
        if target_path.contains(*ch) {
            return Err(format!("目标路径不能包含非法字符: {}", ch));
        }
    }
    
    Ok(())
}

/// 格式化文件大小为人类可读的格式
/// 
/// # 参数
/// * `size` - 文件大小（字节）
/// 
/// # 返回值
/// * 格式化后的大小字符串
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_f, UNITS[unit_index])
    }
}

/// 计算上传进度百分比
/// 
/// # 参数
/// * `uploaded` - 已上传字节数
/// * `total` - 总字节数
/// 
/// # 返回值
/// * 进度百分比（0-100）
pub fn calculate_progress_percentage(uploaded: u64, total: u64) -> f64 {
    if total == 0 {
        return 100.0;
    }
    
    (uploaded as f64 / total as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_upload_url() {
        assert_eq!(
            build_upload_url("http://example.com", "/path/file.txt"),
            "http://example.com/path/file.txt"
        );
        
        assert_eq!(
            build_upload_url("http://example.com/", "path/file.txt"),
            "http://example.com/path/file.txt"
        );
        
        assert_eq!(
            build_upload_url("http://example.com", "path/file.txt"),
            "http://example.com/path/file.txt"
        );
    }

    #[test]
    fn test_validate_target_path() {
        // 有效路径
        assert!(validate_target_path("/valid/path.txt").is_ok());
        assert!(validate_target_path("valid/path.txt").is_ok());
        
        // 无效路径
        assert!(validate_target_path("").is_err());
        assert!(validate_target_path("../invalid/path.txt").is_err());
        assert!(validate_target_path("invalid<path.txt").is_err());
        assert!(validate_target_path("invalid>path.txt").is_err());
        assert!(validate_target_path("invalid:path.txt").is_err());
        assert!(validate_target_path("invalid\"path.txt").is_err());
        assert!(validate_target_path("invalid|path.txt").is_err());
        assert!(validate_target_path("invalid?path.txt").is_err());
        assert!(validate_target_path("invalid*path.txt").is_err());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_file_size(1024_u64.pow(4)), "1.00 TB");
    }

    #[test]
    fn test_calculate_progress_percentage() {
        assert_eq!(calculate_progress_percentage(0, 100), 0.0);
        assert_eq!(calculate_progress_percentage(50, 100), 50.0);
        assert_eq!(calculate_progress_percentage(100, 100), 100.0);
        assert_eq!(calculate_progress_percentage(0, 0), 100.0); // 特殊情况
        
        // 测试小数精度
        let progress = calculate_progress_percentage(33, 100);
        assert!((progress - 33.0).abs() < f64::EPSILON);
    }
}
