use std::ffi::OsString;
use std::fs::Metadata;
use std::path::PathBuf;
use tokio::fs::metadata;

#[derive(Clone, Debug)]
pub struct LocalFileData {
    pub path: PathBuf, // 本地完整路径
    pub file_name: OsString,
    pub is_dir: bool,
}

impl LocalFileData {
    pub fn new(local_path: &str) -> Result<Self, String> {
        let local_path = PathBuf::from(local_path);

        if !local_path.exists() {
            return Err("该路径不存在".to_string());
        }

        let is_dir = local_path.is_dir();

        let file_name = local_path
            .file_name()
            .map(|p| p.to_owned())
            .unwrap_or_else(|| OsString::new());

        Ok(Self { path: local_path, is_dir, file_name })
    }

    pub async fn get_meta_data(&self) -> Result<Option<Metadata>, String> {
        if !self.is_dir {
            let meta = metadata(&self.path)
                .await
                .map_err(|e| format!("读取文件元信息失败: {}", e))?;

            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }
}
