use crate::local_file::structs::local_file::LocalFile;
use std::path::PathBuf;
use tokio::fs;

/// 获取本地文件夹中的所有文件和子目录，不做递归
///
/// # 参数
///
/// * `dir_path` - 目录的绝对路径
///
/// # 返回值
///
/// - 成功时返回 `Vec<LocalFile>` - 包含目录中所有文件和子目录的 LocalFile 对象列表
/// - 失败时返回错误信息字符串
///
/// # 示例
///
/// ```rust,no_run
/// use webdav_client::local_file::public::get_local_folders::get_local_folders;
///
/// #[tokio::main]
/// async fn main() -> Result<(), String> {
///     let files = get_local_folders("/path/to/directory").await?;
///
///     println!("找到 {} 个文件/目录", files.len());
///     for file in files {
///         let state = file.get_reactive_state();
///         let name = state.get_reactive_name().watch();
///         println!("  - {}", name.borrow().as_ref().unwrap_or(&"未知".to_string()));
///     }
///
///     Ok(())
/// }
/// ```
///
/// # 注意
///
/// - 此函数只读取一层目录，不会递归读取子目录中的内容
/// - 返回的列表包含文件和子目录（作为 LocalFile 对象）
/// - 如果目录不存在或无法访问，会返回错误
pub async fn get_local_folders(dir_path: &str) -> Result<Vec<LocalFile>, String> {
    let mut local_files = Vec::new();
    let path = PathBuf::from(dir_path);

    // 检查路径是否存在
    if !path.exists() {
        return Err(format!("路径不存在: {}", dir_path));
    }

    // 检查是否为目录
    let metadata = fs::metadata(&path)
        .await
        .map_err(|e| format!("获取路径元数据失败: {}", e))?;

    if !metadata.is_dir() {
        return Err(format!("路径不是目录: {}", dir_path));
    }

    // 读取目录
    let mut entries = fs::read_dir(&path)
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?;

    // 遍历目录项（不递归）
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取目录项失败: {}", e))?
    {
        let entry_path = entry.path();

        // 将路径转换为字符串
        let path_str = entry_path
            .to_str()
            .ok_or_else(|| format!("路径转换失败: {:?}", entry_path))?;

        // 创建 LocalFile（无论是文件还是目录）
        match LocalFile::new(path_str).await {
            Ok(local_file) => {
                local_files.push(local_file);
            }
            Err(e) => {
                // 记录错误但继续处理其他文件
                eprintln!("警告: 无法创建 LocalFile for {}: {}", path_str, e);
            }
        }
    }

    Ok(local_files)
}