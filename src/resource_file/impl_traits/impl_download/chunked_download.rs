pub mod black_list;
pub(crate) mod file;
pub(crate) mod http_stream;
pub(crate) mod task;

use crate::global_config::global_config::GlobalConfig;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use crate::resource_file::impl_traits::impl_download::chunked_download::file::{computed_semaphore_count, get_local_file_size, open_file, GetLocalFileSizeError, OpenFileError};
use crate::resource_file::impl_traits::impl_download::chunked_download::task::{build_download_tasks, join_all_and_handle_result, BuildDownloadTasksError, DownloadTaskArgs, JoinAllAndHandleResultError};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

pub enum LocalFileDownloadState {
    /// 已完成
    Downloaded,
    /// 未完成
    Incomplete(u64),
}

#[derive(Debug, Error)]
pub enum SetInitialProgressError {
    #[error("更新 download_bytes 出错: {0}")]
    UpdateFieldError(String),
}

pub fn set_initial_progress(
    inner_state: &ResourceFileProperty,
    start: u64,
) -> Result<(), SetInitialProgressError> {
    inner_state
        .download_bytes
        .update_field(|download_bytes| {
            *download_bytes = start as usize;
        })
        .map_err(|e| {
            SetInitialProgressError::UpdateFieldError(e.to_string())
        })?;

    Ok(())
}

pub struct ChunkedDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) http_client: Client,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: ResourceFileProperty,
    pub(crate) inner_config: ResourceConfig,
}

#[derive(Debug, Error)]
pub enum ChunkedDownloadError {
    #[error("文件大小未知，无法分片下载 {0}")]
    UnknownFileSize(String),

    #[error(transparent)]
    GetLocalFileSizeError(#[from] GetLocalFileSizeError),

    #[error(transparent)]
    SetInitialProgressError(#[from] SetInitialProgressError),

    #[error(transparent)]
    BuildDownloadTasksError(#[from] BuildDownloadTasksError),

    #[error(transparent)]
    JoinAllAndHandleResultError(#[from] JoinAllAndHandleResultError),

    #[error(transparent)]
    OpenFileError(#[from] OpenFileError),

    #[error("文件 flush 出错: {0}")]
    FlushError(String),
}

pub async fn chunked_download(
    args: ChunkedDownloadArgs,
) -> Result<(), ChunkedDownloadError> {
    let total_size = args.resource_file_data.size.ok_or_else(|| {
        ChunkedDownloadError::UnknownFileSize(
            args.resource_file_data.absolute_path.clone(),
        )
    })?;

    // 本地文件已下载的大小
    let local_file_downloaded_size =
        get_local_file_size(&args.save_absolute_path, total_size)
            .await
            .map_err(|e| ChunkedDownloadError::GetLocalFileSizeError(e))?;

    let start = match local_file_downloaded_size {
        LocalFileDownloadState::Downloaded => return Ok(()),
        LocalFileDownloadState::Incomplete(size) => size,
    };

    set_initial_progress(&args.inner_state, start)?;

    // 打开文件（续传时用 append + write）
    let file = open_file(&args.save_absolute_path).await?;

    let thread_count =
        computed_semaphore_count(args.resource_file_data.size); // 计算线程数

    let semaphore = Arc::new(Semaphore::new(thread_count));

    let download_task_args = DownloadTaskArgs {
        http_client: &args.http_client,
        file_url: &args.resource_file_data.absolute_path,
        semaphore,
        start,
        total_size,
        file,
        inner_state: &args.inner_state,
        global_config: args.global_config,
        inner_config: args.inner_config,
    };

    let (tasks, mut args) = build_download_tasks(download_task_args)
        .await
        .map_err(|e| ChunkedDownloadError::BuildDownloadTasksError(e))?;

    join_all_and_handle_result(tasks).await?;

    // 关闭文件句柄访问
    args.file
        .flush()
        .await
        .map_err(|e| e.to_string())
        .map_err(|e| ChunkedDownloadError::FlushError(e))?;

    Ok(())
}
