use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use std::path::Path;

/// 获取文件大小
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * `Ok(u64)` - 文件大小（字节）
/// * `Err(String)` - 错误信息
pub async fn get_file_size<P: AsRef<Path>>(file_path: P) -> Result<u64, String> {
    let metadata = tokio::fs::metadata(file_path)
        .await
        .map_err(|e| format!("获取文件大小失败: {}", e))?;
    
    Ok(metadata.len())
}

/// 打开文件用于读取
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * `Ok(File)` - 打开的文件句柄
/// * `Err(String)` - 错误信息
pub async fn open_file_for_read<P: AsRef<Path>>(file_path: P) -> Result<File, String> {
    File::open(file_path)
        .await
        .map_err(|e| format!("打开文件失败: {}", e))
}

/// 从文件中读取指定范围的数据
/// 
/// # 参数
/// * `file` - 文件句柄
/// * `start` - 起始位置（字节）
/// * `size` - 读取大小（字节）
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 读取的数据
/// * `Err(String)` - 错误信息
pub async fn read_file_chunk(
    file: &mut File,
    start: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    // 定位到起始位置
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|e| format!("文件定位失败: {}", e))?;
    
    // 读取数据
    let mut buffer = vec![0u8; size];
    let bytes_read = file.read(&mut buffer)
        .await
        .map_err(|e| format!("读取文件数据失败: {}", e))?;
    
    // 调整缓冲区大小
    buffer.truncate(bytes_read);
    Ok(buffer)
}

/// 读取整个文件内容
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * `Ok(Vec<u8>)` - 文件内容
/// * `Err(String)` - 错误信息
pub async fn read_entire_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<u8>, String> {
    tokio::fs::read(file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))
}

/// 检查文件是否存在
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * `true` - 文件存在
/// * `false` - 文件不存在
pub async fn file_exists<P: AsRef<Path>>(file_path: P) -> bool {
    tokio::fs::metadata(file_path).await.is_ok()
}

/// 检查路径是否为文件（而不是目录）
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回值
/// * `Ok(true)` - 是文件
/// * `Ok(false)` - 不是文件（可能是目录）
/// * `Err(String)` - 错误信息
pub async fn is_file<P: AsRef<Path>>(file_path: P) -> Result<bool, String> {
    let metadata = tokio::fs::metadata(file_path)
        .await
        .map_err(|e| format!("获取文件元数据失败: {}", e))?;
    
    Ok(metadata.is_file())
}

/// 计算文件的分片数量
/// 
/// # 参数
/// * `file_size` - 文件大小（字节）
/// * `chunk_size` - 分片大小（字节）
/// 
/// # 返回值
/// * 分片数量
pub fn calculate_chunk_count(file_size: u64, chunk_size: u64) -> usize {
    ((file_size + chunk_size - 1) / chunk_size) as usize
}

/// 计算指定分片的起始位置和大小
/// 
/// # 参数
/// * `chunk_index` - 分片索引（从0开始）
/// * `chunk_size` - 分片大小（字节）
/// * `total_size` - 文件总大小（字节）
/// 
/// # 返回值
/// * `(start, size)` - 起始位置和实际大小
pub fn calculate_chunk_range(
    chunk_index: usize,
    chunk_size: u64,
    total_size: u64,
) -> (u64, u64) {
    let start = chunk_index as u64 * chunk_size;
    let end = std::cmp::min(start + chunk_size, total_size);
    let size = end - start;
    (start, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_file_operations() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is a test file.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        let file_path = temp_file.path();
        
        // 测试文件大小
        let size = get_file_size(file_path).await.unwrap();
        assert_eq!(size, test_data.len() as u64);
        
        // 测试文件存在性
        assert!(file_exists(file_path).await);
        
        // 测试是否为文件
        assert!(is_file(file_path).await.unwrap());
        
        // 测试读取整个文件
        let content = read_entire_file(file_path).await.unwrap();
        assert_eq!(content, test_data);
    }

    #[tokio::test]
    async fn test_chunk_operations() {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();
        
        let file_path = temp_file.path();
        let mut file = open_file_for_read(file_path).await.unwrap();
        
        // 测试读取分片
        let chunk = read_file_chunk(&mut file, 5, 10).await.unwrap();
        assert_eq!(chunk, b"56789ABCDE");
        
        // 测试读取另一个分片
        let chunk2 = read_file_chunk(&mut file, 0, 5).await.unwrap();
        assert_eq!(chunk2, b"01234");
    }

    #[test]
    fn test_chunk_calculations() {
        // 测试分片数量计算
        assert_eq!(calculate_chunk_count(100, 30), 4); // 100/30 = 3.33... -> 4
        assert_eq!(calculate_chunk_count(90, 30), 3);  // 90/30 = 3
        assert_eq!(calculate_chunk_count(1, 30), 1);   // 1/30 = 0.03... -> 1
        
        // 测试分片范围计算
        assert_eq!(calculate_chunk_range(0, 30, 100), (0, 30));   // 第1片: 0-29
        assert_eq!(calculate_chunk_range(1, 30, 100), (30, 30));  // 第2片: 30-59
        assert_eq!(calculate_chunk_range(2, 30, 100), (60, 30));  // 第3片: 60-89
        assert_eq!(calculate_chunk_range(3, 30, 100), (90, 10));  // 第4片: 90-99 (只有10字节)
    }
}
