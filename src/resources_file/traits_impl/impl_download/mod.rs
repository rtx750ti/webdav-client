mod chunked_download;
mod handle_download;
mod not_chunked_download;

use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use crate::resources_file::traits_impl::impl_download::handle_download::{handle_download, HandleDownloadArgs};

/// 预处理保存文件路径
fn preprocessing_save_path(
    resource_file_data: Arc<ResourceFileData>,
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
    ) -> Result<Arc<Self>, String> {
        let save_absolute_path =
            preprocessing_save_path(self.get_data(), save_absolute_path)
                .map_err(|e| {
                format!("[preprocessing_save_path] {}", e.to_string())
            })?;

        let http_client = self.get_http_client();

        let handle_download_args = HandleDownloadArgs {
            resource_file_data: self.get_data(),
            save_absolute_path,
            http_client: http_client.clone(),
            global_config: self.get_global_config(),
            inner_state: self.get_reactive_state(),
            inner_config: self.get_reactive_config(),
        };

        handle_download(handle_download_args)
            .await
            .map_err(|e| format!("[handle_download] {}", e.to_string()))?;
        Ok(Arc::new(self))
    }
}
