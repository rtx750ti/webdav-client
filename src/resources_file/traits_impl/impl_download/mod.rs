mod chunked_download;
mod handle_download;
mod not_chunked_download;

use crate::reactive::ReactiveProperty;
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;
use crate::resources_file::traits_impl::impl_download::handle_download::{
    HandleDownloadArgs, handle_download,
};
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

fn watch_self_reactive_property(
    inner_config: ReactiveConfig,
    inner_state: ReactiveFileProperty,
) -> JoinHandle<()> {
    let mut config_watcher = inner_config.watch();
    let mut download_bytes_watcher = inner_state.download_bytes.watch();
    let file_lock = inner_state.file_lock;

    let current_file_name = inner_state.name.get_current();

    tokio::spawn(async move {
        loop {
            if let Some(file_lock) = file_lock.get_current() {
                if !file_lock {
                    break;
                }
            } else {
                println!(
                    "[watch_self_reactive_property] 文件[{:?}]无法获取锁",
                    current_file_name
                );
                break;
            }

            select! {
                res = config_watcher.changed() => {
                    match res {
                        Ok(config) => {
                            println!("文件[{:?}]配置更新{:?}",current_file_name,config);
                        }
                        Err(err) => {
                            println!("文件[{:?}]配置监听器异常:{}",current_file_name,err);
                            break;
                        },
                    }
                },
                res = download_bytes_watcher.changed() => {
                    match res {
                        Ok(_) => {}
                        Err(err) => {
                            println!("文件[{:?}]下载出错:{}",current_file_name,err);
                            break;
                        },
                    }
                }
            }
        }
    })
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
           async || -> Result<(JoinHandle<()>, JoinHandle<()>), String> {
                // 以下顺序不能乱
                // 1、监听文件锁
                let watching_file_lock =
                    watch_self_file_lock(self.get_reactive_state());

                // 2、获取资源文件锁
                self.lock_file(false).await?; // 这里可能获取失败，如果获取失败就不下载，交给使用者来处理是否继续

                // 3、创建响应式监听任务，这个顺序不能错，必须得让锁为true才行，如果不提前监听，则上一步无法加锁
                let watching_property_task = watch_self_reactive_property(
                    self.get_reactive_config(),
                    self.get_reactive_state(),
                );

                Ok((watching_file_lock, watching_property_task))
            };

        let (watching_file_lock, watching_property_task) =
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
            // 2、不管成功和失败，必须销毁监听任务
            watching_property_task.abort();
            // 3、最后再销毁文件锁监听器
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
