use std::fs::FileType;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::fs::File;

#[derive(Debug)]
pub struct LocalFileData {
    file: File,
    path: PathBuf,
}

pub struct FileMeta {
    pub name: String,
    pub path: PathBuf,
    pub len: u64,
    pub is_dir: bool,
    pub readonly: bool,
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
