pub(crate) mod chunked_download;
pub(crate) mod handle_download;
pub(crate) mod not_chunked_download;

use crate::resource_file::impl_traits::impl_download::handle_download::{
    HandleDownloadArgs, handle_download,
};
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use crate::resource_file::structs::resources_file::ResourcesFile;
use crate::resource_file::traits::download::{Download, DownloadError};
use async_trait::async_trait;
use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PreprocessingSavePathError {
    #[error("PathBuf::from_str 失败: {0}")]
    FromStrError(#[from] Infallible),
}

/// 预处理保存文件路径
fn preprocessing_save_path(
    resource_file_data: Arc<ResourceFileData>,
    save_absolute_path: &str,
) -> Result<PathBuf, PreprocessingSavePathError> {
    // 预处理保存文件的完整路径
    let path = PathBuf::from_str(save_absolute_path)?;

    if resource_file_data.is_dir {
        Ok(path)
    } else {
        Ok(path.join(&resource_file_data.name))
    }
}

#[derive(Debug, Error)]
pub enum HandleMountedError {
    #[error("lock_file 失败: {0}")]
    LockFileError(String),
}

#[derive(Debug, Error)]
pub enum HandleUnmountedError {
    #[error("unlock_file 失败: {0}")]
    UnlockFileError(String),
}

#[async_trait]
impl Download for ResourcesFile {
    async fn download(
        self,
        save_absolute_path: &str,
    ) -> Result<Arc<Self>, DownloadError> {
        let handle_mounted = async || -> Result<(), HandleMountedError> {
            // 获取资源文件锁
            self.lock_file(false)
                .await
                .map_err(|e| HandleMountedError::LockFileError(e))?; // 这里可能获取失败，如果获取失败就不下载，交给使用者来处理是否继续

            Ok(())
        };

        handle_mounted().await?;

        let save_absolute_path =
            preprocessing_save_path(self.get_data(), save_absolute_path)?;

        let http_client = self.get_http_client();

        let handle_download_args = HandleDownloadArgs {
            resource_file_data: self.get_data(),
            save_absolute_path,
            http_client: http_client.clone(),
            global_config: self.get_global_config(),
            inner_state: self.get_reactive_state(),
            inner_config: self.get_reactive_config(),
        };

        let download_result = handle_download(handle_download_args).await;

        let handle_unmounted =
            async || -> Result<(), HandleUnmountedError> {
                // 以下顺序不能乱
                // 解锁资源文件
                self.unlock_file(false).await.map_err(|e| {
                    HandleUnmountedError::UnlockFileError(e)
                })?;

                Ok(())
            };

        handle_unmounted().await?;

        // 处理可能的失败结果
        download_result?;

        Ok(Arc::new(self))
    }
}
