use std::cmp::min;
use std::sync::Arc;
use futures_util::future::join_all;
use reqwest::Client;
use tokio::fs::File;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits_impl::impl_download::chunked_download::CHUNK_SIZE;
use crate::resources_file::traits_impl::impl_download::chunked_download::file::clone_file_handle;
use crate::resources_file::traits_impl::impl_download::chunked_download::http::{download_range_file, DownloadRangeFileArgs};

pub fn computed_range_header(total_size: u64, start: u64) -> String {
    let end = min(start + CHUNK_SIZE - 1, total_size - 1);
    let range_header_str = format!("bytes={}-{}", start, end);

    range_header_str
}

pub struct DownloadTaskArgs<'a> {
    pub(crate) http_client: &'a Client,
    pub file_url: &'a str,
    pub semaphore: Arc<Semaphore>,
    pub start: u64,
    pub total_size: u64,
    pub file: File,
    pub inner_state: &'a ReactiveFileProperty,
}

pub type DownloadTasks = Vec<JoinHandle<Result<(), String>>>;

/// 构建下载任务
pub async fn build_download_tasks<'a>(
    mut args: DownloadTaskArgs<'a>,
) -> Result<(DownloadTasks, DownloadTaskArgs<'a>), String> {
    let mut tasks = vec![];

    // 将下载任务分配到并发线程
    while args.start < args.total_size {
        let range_header_str =
            computed_range_header(args.total_size, args.start);

        let semaphore = Arc::clone(&args.semaphore); // 克隆一个Arc指针，传递给异步任务

        let http_client = args.http_client.clone();
        let file_url = args.file_url.to_string();

        let mut file = clone_file_handle(&args.file).await?;

        let start = args.start;

        let task = tokio::task::spawn(async move {
            // 使用Semaphore来限制并发数量
            let permit = semaphore.acquire().await;

            return match permit {
                Ok(_) => {
                    let download_range_file_args = DownloadRangeFileArgs {
                        http_client: &http_client,
                        range_header_str: &range_header_str,
                        file_url: &file_url,
                        file: &mut file,
                        start,
                    };

                    download_range_file(download_range_file_args).await?;

                    Ok(())
                }
                Err(err) => {
                    Err(format!("无法获取并发许可: {}", err.to_string()))
                }
            };
        });

        tasks.push(task);
        args.start += CHUNK_SIZE;
    }

    Ok((tasks, args))
}

pub async fn join_all_and_handle_result(
    tasks: DownloadTasks,
) -> Result<(), String> {
    // 等待所有任务完成
    let results = join_all(tasks).await;

    // 检查任务的结果
    for result in results {
        result
            .map_err(|e| format!("[chunked_download] {}", e.to_string()))?
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
