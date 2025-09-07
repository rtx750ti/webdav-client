use crate::client::traits::folder::TResourcesFileCollection;
use crate::download_config::DownloadConfig;
use crate::downloader::Downloader;
use crate::downloader::enums::download_task::DownloadTask;
use crate::downloader::traits::batch_download::BatchDownload;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use futures_util::StreamExt;
use futures_util::stream::FuturesUnordered;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[async_trait]
impl BatchDownload for Downloader {
    async fn download(
        &self,
        save_absolute_path: &str,
        download_task: DownloadTask,
        download_config: &DownloadConfig,
    ) -> Result<(), String> {
        match download_task {
            DownloadTask::ResourcesFileCollectionList(folders_result) => {
                // 扁平化数组
                let flat_list: TResourcesFileCollection =
                    folders_result.into_iter().flatten().collect();

                let semaphore = Arc::new(Semaphore::new(
                    download_config.max_thread_count as usize,
                ));

                // 并发下载
                let mut tasks = FuturesUnordered::new();

                for resources_file in flat_list {
                    let permit =
                        semaphore.clone().acquire_owned().await.unwrap();
                    let save_path = save_absolute_path.to_string();
                    let config = download_config.clone(); // 需要实现 Clone trait
                    tasks.push(tokio::spawn(async move {
                        let result = resources_file.download(&save_path, &config).await;
                        drop(permit); // 释放信号量
                        result.map_err(|e| {
                            format!(
                                "[BatchDownload->download_task->ResourcesFileCollectionList] {}",
                                e.to_string()
                            )
                        })
                    }));
                }

                // 收集结果
                while let Some(result) = tasks.next().await {
                    match result {
                        Ok(Err(e)) => return Err(e),
                        Ok(Ok(_)) => continue,
                        Err(e) => {
                            return Err(format!("任务执行失败: {}", e));
                        }
                    }
                }

                Ok(())
            }
            DownloadTask::ResourcesFile(resources_file) => resources_file
                .download(save_absolute_path, download_config)
                .await
                .map(|_| ())
                .map_err(|e| {
                    format!(
                        "[BatchDownload->download_task->ResourcesFile] {}",
                        e.to_string()
                    )
                }),
        }
    }
}
