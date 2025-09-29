use crate::global_config::GlobalConfig;
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::traits_impl::impl_download::chunked_download::file::clone_file_handle;
use crate::resources_file::traits_impl::impl_download::chunked_download::http_stream::{download_range_file, DownloadRangeFileArgs};
use crate::resources_file::traits_impl::impl_download::chunked_download::CHUNK_SIZE;
use futures_util::future::join_all;
use reqwest::Client;
use std::cmp::min;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

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
    pub global_config: GlobalConfig,
    pub inner_config: ReactiveConfig,
}

pub type DownloadTasks = Vec<JoinHandle<Result<(), String>>>;

struct DownloadTaskContext {
    pub http_client: Client,
    pub file_url: String,
    pub range_header_str: String,
    pub file: File,
    pub start: u64,
    pub inner_state: ReactiveFileProperty,
    pub global_config: GlobalConfig,
    pub inner_config: ReactiveConfig,
}

impl DownloadTaskContext {
    fn into_range_file_args<'a>(&'a mut self) -> DownloadRangeFileArgs<'a> {
        DownloadRangeFileArgs {
            http_client: &self.http_client,
            range_header_str: &self.range_header_str,
            file_url: &self.file_url,
            file: &mut self.file,
            start: self.start,
            inner_state: self.inner_state.clone(),
            global_config: self.global_config.clone(),
            inner_config: self.inner_config.clone(),
        }
    }
}


/// 构建下载任务
pub async fn build_download_tasks<'a>(
    mut args: DownloadTaskArgs<'a>,
) -> Result<(DownloadTasks, DownloadTaskArgs<'a>), String> {
    let mut tasks = vec![];

    // 将下载任务分配到并发线程
    while args.start < args.total_size {
        let range_header_str =
            computed_range_header(args.total_size, args.start);

        let cloned_file_handle = clone_file_handle(&args.file).await?;

        let mut context = DownloadTaskContext {
            http_client: args.http_client.clone(),
            file_url: args.file_url.to_string(),
            range_header_str,
            file: cloned_file_handle,
            start: args.start,
            inner_state: args.inner_state.clone(),
            global_config: args.global_config.clone(),
            inner_config: args.inner_config.clone(),
        };

        let semaphore = Arc::clone(&args.semaphore);

        let task = tokio::task::spawn(async move {
            // 使用Semaphore来限制并发数量
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|e| format!("无法获取并发许可: {}", e))?;

            let download_range_file_args = context.into_range_file_args();

            download_range_file(download_range_file_args).await?;

            Ok(())
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
