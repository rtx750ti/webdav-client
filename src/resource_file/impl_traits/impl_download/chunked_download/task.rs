use crate::global_config::global_config::GlobalConfig;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use crate::resource_file::impl_traits::impl_download::chunked_download::file::{clone_file_handle, CloneFileHandleError};
use crate::resource_file::impl_traits::impl_download::chunked_download::http_stream::{download_range_file, DownloadRangeFileArgs};
use crate::resource_file::impl_traits::impl_download::chunked_download::CHUNK_SIZE;
use futures_util::future::join_all;
use reqwest::Client;
use std::cmp::min;
use std::sync::Arc;
use thiserror::Error;
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
    pub inner_state: &'a ResourceFileProperty,
    pub global_config: GlobalConfig,
    pub inner_config: ResourceConfig,
}

pub type DownloadTasks = Vec<JoinHandle<Result<(), String>>>;

struct DownloadTaskContext {
    pub http_client: Client,
    pub file_url: String,
    pub range_header_str: String,
    pub file: File,
    pub start: u64,
    pub inner_state: ResourceFileProperty,
    pub global_config: GlobalConfig,
    pub inner_config: ResourceConfig,
}

impl DownloadTaskContext {
    fn into_range_file_args<'a>(
        &'a mut self,
    ) -> DownloadRangeFileArgs<'a> {
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

#[derive(Debug, Error)]
pub enum BuildDownloadTasksError {
    #[error("clone_file_handle 出错: {0}")]
    CloneFileHandleError(#[from] CloneFileHandleError),

    #[error("无法获取并发许可: {0}")]
    AcquirePermitError(String),

    #[error("download_range_file 出错: {0}")]
    DownloadRangeFileError(String),
}

/// 构建下载任务
pub async fn build_download_tasks<'a>(
    mut args: DownloadTaskArgs<'a>,
) -> Result<(DownloadTasks, DownloadTaskArgs<'a>), BuildDownloadTasksError>
{
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

#[derive(Debug, Error)]
pub enum JoinAllAndHandleResultError {
    #[error("任务 Join 出错: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("任务执行出错: {0}")]
    TaskError(String),
}

pub async fn join_all_and_handle_result(
    tasks: DownloadTasks,
) -> Result<(), JoinAllAndHandleResultError> {
    // 等待所有任务完成
    let results = join_all(tasks).await;

    // 检查任务的结果
    for result in results {
        // JoinError
        let inner = result?;

        // 任务返回的 Err(String)
        inner.map_err(|e| {
            JoinAllAndHandleResultError::TaskError(e.to_string())
        })?;
    }

    Ok(())
}
