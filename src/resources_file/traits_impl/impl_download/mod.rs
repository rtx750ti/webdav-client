mod download;
mod restart;
mod start;
mod stop;

use std::path::PathBuf;
use std::str::FromStr;
use crate::resources_file::structs::download_config::DownloadConfig;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use std::sync::Arc;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits_impl::impl_download::download::download_file::handle_download;

/// 预处理保存文件路径
fn preprocessing_save_path(
    resource_file_data: &ResourceFileData,
    save_absolute_path: &str,
) -> Result<PathBuf, String> {
    // 预处理保存文件的完整路径
    let path = PathBuf::from_str(save_absolute_path)
        .map_err(|e| format!("[from_str] {}", e.to_string()))?;

    if resource_file_data.is_dir {
        Ok(path)
    } else {
        Ok(path.join(&resource_file_data.name))
    }
}

#[async_trait]
impl Download for ResourcesFile {
    async fn download(
        self,
        save_absolute_path: &str,
        download_config: &DownloadConfig,
    ) -> Result<Arc<Self>, String> {
        let save_absolute_path =
            preprocessing_save_path(self.get_data(), save_absolute_path)
                .map_err(|e| {
                format!("[preprocessing_save_path] {}", e.to_string())
            })?;

        let http_client = self.get_http_client();
        handle_download(
            self.get_data(),
            &save_absolute_path,
            http_client,
            download_config,
        )
        .await
        .map_err(|e| format!("[handle_download] {}", e.to_string()))?;
        Ok(Arc::new(self))
    }

    async fn stop(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }

    async fn start(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }

    async fn restart(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }
}
