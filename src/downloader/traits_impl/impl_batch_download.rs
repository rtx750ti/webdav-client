use crate::client::traits::folder::TResourcesFileCollection;
use crate::downloader::Downloader;
use crate::downloader::enums::download_task::DownloadTask;
use crate::downloader::traits::batch_download::BatchDownload;
use crate::resources_file::structs::download_config::DownloadConfig;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use futures_util::StreamExt;
use futures_util::stream::FuturesUnordered;

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

                // 并发下载
                let mut tasks = FuturesUnordered::new();

                for resources_file in flat_list {
                    let save_path = save_absolute_path.to_string();
                    let config = download_config;
                    tasks.push(async move {
                        resources_file
                            .download(&save_path, &config)
                            .await
                            .map_err(|e| {
                                format!(
                                    "[BatchDownload->download_task->ResourcesFileCollectionList] {}",
                                    e.to_string()
                                )
                            })
                    });
                }

                // 收集结果
                while let Some(result) = tasks.next().await {
                    if let Err(e) = result {
                        return Err(e);
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
