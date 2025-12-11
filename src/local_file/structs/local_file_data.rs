use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::fs::File;

#[derive(Debug)]
pub struct LocalFileData {
    /// 读取到的文件句柄
    pub file: File,
    /// 文件路径
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct FileMeta {
    /// 文件名
    pub name: String,
    /// 路径
    pub path: PathBuf,
    /// 大小（Byte）
    pub len: u64,
    /// 是否是文件夹
    pub is_dir: bool,
    /// 是否只读
    pub readonly: bool,
    /// 最后的修改时间戳（精确到毫秒）
    pub modified: Option<SystemTime>,
}

async fn open_file(absolute_path: &PathBuf) -> Result<File, String> {
    // 打开文件（续传时用 append + write）
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(absolute_path)
        .await
        .map_err(|e| e.to_string())?;

    let meta = file.metadata().await.map_err(|e| e.to_string())?;

    Ok(file)
}

impl LocalFileData {
    pub async fn new(absolute_path: &PathBuf) -> Result<Self, String> {
        let file =
            open_file(absolute_path).await.map_err(|e| e.to_string())?;

        Ok(Self { file, path: absolute_path.clone() })
    }

    pub async fn get_meta(&self) -> Result<FileMeta, String> {
        let meta =
            self.file.metadata().await.map_err(|e| e.to_string())?;

        let modified = meta.modified().ok();

        let file_name = self
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(FileMeta {
            name: file_name.to_string(),
            len: meta.len(),
            is_dir: meta.is_dir(),
            readonly: meta.permissions().readonly(),
            path: self.path.clone(),
            modified,
        })
    }
}
