/// 分片上传黑名单
/// 
/// 某些文件类型或路径不适合分片上传，例如：
/// - 小文件（分片开销大于收益）
/// - 特定格式文件（可能不支持分片）
/// - 临时文件或配置文件
pub const CHUNKED_UPLOAD_BLACKLIST: [&str; 6] = [
    ".tmp",      // 临时文件
    ".log",      // 日志文件
    ".config",   // 配置文件
    ".ini",      // 配置文件
    ".json",     // JSON配置文件
    ".xml",      // XML配置文件
];

/// 检查文件是否在分片上传黑名单中
/// 
/// # 参数
/// * `target_path` - 目标文件路径
/// 
/// # 返回值
/// * `true` - 文件在黑名单中，不应使用分片上传
/// * `false` - 文件不在黑名单中，可以使用分片上传
pub fn is_chunked_upload_blacklisted(target_path: &str) -> bool {
    CHUNKED_UPLOAD_BLACKLIST
        .iter()
        .any(|pattern| target_path.to_lowercase().contains(pattern))
}

/// 检查文件扩展名是否在黑名单中
/// 
/// # 参数
/// * `file_extension` - 文件扩展名（包含点号，如 ".txt"）
/// 
/// # 返回值
/// * `true` - 扩展名在黑名单中
/// * `false` - 扩展名不在黑名单中
pub fn is_extension_blacklisted(file_extension: &str) -> bool {
    let ext = file_extension.to_lowercase();
    CHUNKED_UPLOAD_BLACKLIST
        .iter()
        .any(|pattern| ext == *pattern)
}

/// 根据文件大小和路径决定是否应该使用分片上传
/// 
/// # 参数
/// * `target_path` - 目标文件路径
/// * `file_size` - 文件大小（字节）
/// * `chunk_threshold` - 分片阈值（字节）
/// 
/// # 返回值
/// * `true` - 应该使用分片上传
/// * `false` - 应该使用简单上传
pub fn should_use_chunked_upload(
    target_path: &str,
    file_size: u64,
    chunk_threshold: u64,
) -> bool {
    // 检查黑名单
    if is_chunked_upload_blacklisted(target_path) {
        return false;
    }
    
    // 检查文件大小
    file_size > chunk_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_upload_blacklist() {
        // 测试黑名单文件
        assert!(is_chunked_upload_blacklisted("/path/to/file.tmp"));
        assert!(is_chunked_upload_blacklisted("/path/to/app.log"));
        assert!(is_chunked_upload_blacklisted("/path/to/settings.config"));
        assert!(is_chunked_upload_blacklisted("/path/to/app.ini"));
        assert!(is_chunked_upload_blacklisted("/path/to/data.json"));
        assert!(is_chunked_upload_blacklisted("/path/to/config.xml"));
        
        // 测试非黑名单文件
        assert!(!is_chunked_upload_blacklisted("/path/to/document.pdf"));
        assert!(!is_chunked_upload_blacklisted("/path/to/video.mp4"));
        assert!(!is_chunked_upload_blacklisted("/path/to/archive.zip"));
    }

    #[test]
    fn test_extension_blacklist() {
        // 测试黑名单扩展名
        assert!(is_extension_blacklisted(".tmp"));
        assert!(is_extension_blacklisted(".log"));
        assert!(is_extension_blacklisted(".config"));
        assert!(is_extension_blacklisted(".TMP")); // 大小写不敏感
        
        // 测试非黑名单扩展名
        assert!(!is_extension_blacklisted(".pdf"));
        assert!(!is_extension_blacklisted(".mp4"));
        assert!(!is_extension_blacklisted(".zip"));
    }

    #[test]
    fn test_should_use_chunked_upload() {
        let threshold = 5 * 1024 * 1024; // 5MB
        
        // 大文件，不在黑名单 -> 应该分片
        assert!(should_use_chunked_upload("/path/to/large.zip", 10 * 1024 * 1024, threshold));
        
        // 大文件，在黑名单 -> 不应该分片
        assert!(!should_use_chunked_upload("/path/to/large.tmp", 10 * 1024 * 1024, threshold));
        
        // 小文件，不在黑名单 -> 不应该分片
        assert!(!should_use_chunked_upload("/path/to/small.zip", 1024, threshold));
        
        // 小文件，在黑名单 -> 不应该分片
        assert!(!should_use_chunked_upload("/path/to/small.tmp", 1024, threshold));
    }
}
