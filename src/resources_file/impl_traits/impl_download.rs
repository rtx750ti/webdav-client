mod chunked_download;
mod handle_download;
mod not_chunked_download;

use crate::resources_file::impl_traits::impl_download::handle_download::{
    HandleDownloadArgs, handle_download,
};
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::select;
use tokio::task::JoinHandle;

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

fn watch_self_file_lock(
    inner_state: ReactiveFileProperty,
) -> JoinHandle<()> {
    let current_file_name = inner_state.name.get_current();

    let file_lock = inner_state.get_file_lock().clone();

    tokio::spawn(async move {
        while let Ok(file_lock) = file_lock.watch().changed().await {
            println!("文件锁改变[{:?}]→{}", current_file_name, file_lock);
        }
    })
}

#[async_trait]
impl Download for ResourcesFile {
    async fn download(
        self,
        save_absolute_path: &str,
    ) -> Result<Arc<Self>, String> {
        let handle_mounted =
            async || -> Result<(JoinHandle<()>), String> {
                // 以下顺序不能乱
                // 1、监听文件锁
                let watching_file_lock =
                    watch_self_file_lock(self.get_reactive_state());

                // 2、获取资源文件锁
                self.lock_file(false).await?; // 这里可能获取失败，如果获取失败就不下载，交给使用者来处理是否继续

                Ok(watching_file_lock)
            };

        let watching_file_lock =
            handle_mounted().await?;

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

        let download_result = handle_download(handle_download_args).await;

        let handle_unmounted = async || -> Result<(), String> {
            // 以下顺序不能乱
            // 1、解锁资源文件
            self.unlock_file(false).await?;
            // 2、最后再销毁文件锁监听器
            watching_file_lock.abort();

            Ok(())
        };

        handle_unmounted().await?;

        // 处理可能的失败结果
        download_result
            .map_err(|e| format!("[handle_download] {}", e.to_string()))?;

        Ok(Arc::new(self))
    }
}
