use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::fs::File;

/// 本地文件数据枚举
///
/// 支持文件和文件夹两种类型
#[derive(Debug)]
pub enum LocalFileData {
    /// 文件类型，包含文件句柄和路径
    File { file: File, path: PathBuf },
    /// 文件夹类型，只包含路径
    Directory { path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct FileMeta {
    pub name: String,
    pub path: PathBuf,
    pub len: u64,
    pub is_dir: bool,
    pub readonly: bool,
    pub modified: Option<SystemTime>,
}

async fn open_file(absolute_path: &PathBuf) -> Result<File, String> {
    // 打开文件（读写模式）
    let file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(absolute_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(file)
}

impl LocalFileData {
    /// 创建新的 LocalFileData
    ///
    /// 根据路径自动判断是文件还是文件夹
    pub async fn new(absolute_path: &PathBuf) -> Result<Self, String> {
        // 检查路径是否存在
        let metadata = fs::metadata(absolute_path)
            .await
            .map_err(|e| format!("无法获取路径元数据: {}", e))?;

        if metadata.is_dir() {
            // 如果是目录，直接返回 Directory 类型
            Ok(Self::Directory {
                path: absolute_path.clone(),
            })
        } else {
            // 如果是文件，打开文件并返回 File 类型
            let file = open_file(absolute_path).await?;
            Ok(Self::File {
                file,
                path: absolute_path.clone(),
            })
        }
    }

    /// 获取元数据
    pub async fn get_meta(&self) -> Result<FileMeta, String> {
        match self {
            LocalFileData::File { file, path } => {
                let meta = file.metadata().await.map_err(|e| e.to_string())?;
                let modified = meta.modified().ok();
                let file_name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                Ok(FileMeta {
                    name: file_name,
                    len: meta.len(),
                    is_dir: false,
                    readonly: meta.permissions().readonly(),
                    path: path.clone(),
                    modified,
                })
            }
            LocalFileData::Directory { path } => {
                let meta = fs::metadata(path).await.map_err(|e| e.to_string())?;
                let modified = meta.modified().ok();
                let dir_name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                Ok(FileMeta {
                    name: dir_name,
                    len: 0, // 目录大小为 0
                    is_dir: true,
                    readonly: meta.permissions().readonly(),
                    path: path.clone(),
                    modified,
                })
            }
        }
    }

    /// 获取文件路径
    pub fn get_path(&self) -> &PathBuf {
        match self {
            LocalFileData::File { path, .. } => path,
            LocalFileData::Directory { path } => path,
        }
    }

    /// 获取文件引用（仅对文件类型有效）
    pub fn get_file(&self) -> Result<&File, String> {
        match self {
            LocalFileData::File { file, .. } => Ok(file),
            LocalFileData::Directory { .. } => {
                Err("目录类型不支持获取文件句柄".to_string())
            }
        }
    }
}
