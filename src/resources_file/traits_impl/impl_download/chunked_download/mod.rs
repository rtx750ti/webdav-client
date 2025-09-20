mod file;
mod http;
mod task;
pub mod black_list;

use std::path::PathBuf;
use crate::public::utils::handle_file::computed_semaphore_count;
use std::sync::Arc;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use crate::global_config::GlobalConfig;
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits_impl::impl_download::chunked_download::file::{get_local_file_size, open_file};
use crate::resources_file::traits_impl::impl_download::chunked_download::task::{build_download_tasks, join_all_and_handle_result, DownloadTaskArgs};

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

pub enum LocalFileDownloadState {
    /// 已完成
    Downloaded,
    /// 未完成
    Incomplete(u64),
}

pub struct ChunkedDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) http_client: Client,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: ReactiveFileProperty,
    pub(crate) inner_config: ReactiveConfig,
}

pub async fn chunked_download(
    args: ChunkedDownloadArgs,
) -> Result<(), String> {
    let total_size = args.resource_file_data.size.ok_or_else(|| {
        format!(
            "文件大小未知，无法分片下载 {}",
            args.resource_file_data.absolute_path
        )
    })?;

    // 本地文件已下载的大小
    let local_file_downloaded_size =
        get_local_file_size(&args.save_absolute_path, total_size)
            .await
            .map_err(|e| e.to_string())?;

    let start = match local_file_downloaded_size {
        LocalFileDownloadState::Downloaded => return Ok(()),
        LocalFileDownloadState::Incomplete(size) => size,
    };

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
    };

    let (tasks, mut args) =
        build_download_tasks(download_task_args).await?;

    join_all_and_handle_result(tasks).await?;

    // 关闭文件句柄访问
    args.file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}
