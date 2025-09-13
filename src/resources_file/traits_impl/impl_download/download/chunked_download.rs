use crate::download_config::DownloadConfig;
use crate::public::utils::handle_file::computed_semaphore_count;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use futures_util::future::join_all;
use reqwest::Client;
use reqwest::header::RANGE;
use std::cmp::min;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use crate::resources_file::traits::download::TDownloadConfig;

/// 分片黑名单，这些厂商不讲武德，拒绝分片请求，甚至拿1比特数据都要算下载了整个文件的流量
const CHUNKED_DOWNLOAD_BLACKLIST: [&str; 1] =
    ["https://dav.jianguoyun.com/"];

/// 查找地址是否在分片黑名单里
pub fn is_chunked_download_blacklisted(base_url: &str) -> bool {
    CHUNKED_DOWNLOAD_BLACKLIST
        .iter()
        .any(|blacklisted_url| base_url.starts_with(blacklisted_url))
}

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

pub enum LocalFileDownloadState {
    /// 已完成
    Downloaded,
    /// 未完成
    Incomplete(u64),
}

pub async fn get_local_file_size(
    save_absolute_path: &PathBuf,
    total_size: u64,
) -> Result<LocalFileDownloadState, String> {
    // 检查是否已有部分文件（断点续传）
    let mut local_size: u64 = 0;
    if let Ok(meta) = tokio::fs::metadata(save_absolute_path).await {
        local_size = meta.len();
        if local_size > total_size {
            // 本地文件比远程大，说明出错，删掉重新下
            tokio::fs::remove_file(save_absolute_path)
                .await
                .map_err(|e| e.to_string())?;
            local_size = 0;
        } else if local_size == total_size {
            // 已经下载完成
            return Ok(LocalFileDownloadState::Downloaded);
        }
    }

    Ok(LocalFileDownloadState::Incomplete(local_size))
}

struct DownloadTaskOption<'a> {
    http_client: &'a Client,
    file_url: &'a str,
    semaphore: Arc<Semaphore>,
    start: u64,
    total_size: u64,
    file: File,
}

type DownloadTask = Vec<JoinHandle<Result<(), String>>>;

/// 构建下载任务
async fn build_download_tasks<'a>(
    mut download_task_option: DownloadTaskOption<'a>,
) -> Result<(DownloadTask, DownloadTaskOption), String> {
    let mut tasks = vec![];

    // 将下载任务分配到并发线程
    while download_task_option.start < download_task_option.total_size {
        let end = min(
            download_task_option.start + CHUNK_SIZE - 1,
            download_task_option.total_size - 1,
        );
        let range_header =
            format!("bytes={}-{}", download_task_option.start, end);

        let semaphore = Arc::clone(&download_task_option.semaphore); // 克隆一个Arc指针，传递给异步任务

        let task = tokio::task::spawn({
            let http_client = download_task_option.http_client.clone();
            let file_url = download_task_option.file_url.to_string();
            let start = download_task_option.start;
            let end = end;
            let mut file = download_task_option
                .file
                .try_clone()
                .await
                .map_err(|e| e.to_string())?;

            async move {
                // 使用Semaphore来限制并发数量
                let permit = semaphore.acquire().await;

                match permit {
                    Ok(_) => {
                        let resp = http_client
                            .get(&file_url)
                            .header(RANGE, range_header)
                            .send()
                            .await
                            .map_err(|e| e.to_string())?;

                        if !resp.status().is_success() {
                            return Err(format!(
                                "分片下载失败: {} - {}",
                                resp.status(),
                                &file_url
                            ));
                        }

                        let chunk = resp
                            .bytes()
                            .await
                            .map_err(|e| e.to_string())?;

                        file.seek(std::io::SeekFrom::Start(start))
                            .await
                            .map_err(|e| e.to_string())?;
                        file.write_all(&chunk)
                            .await
                            .map_err(|e| e.to_string())?;
                        return Ok(());
                    }
                    Err(err) => {
                        return Err(format!(
                            "无法获取并发许可: {}",
                            err.to_string()
                        ));
                    }
                }
            }
        });

        tasks.push(task);
        download_task_option.start += CHUNK_SIZE;
    }

    Ok((tasks, download_task_option))
}

pub struct ChunkedDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) http_client: Client,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) download_config: TDownloadConfig,
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

    let local_file_download_state =
        get_local_file_size(&args.save_absolute_path, total_size)
            .await
            .map_err(|e| e.to_string())?;

    let mut start = total_size;

    let _ = start; // 读取一次，避免未读警告

    match local_file_download_state {
        LocalFileDownloadState::Downloaded => return Ok(()), // 以后可以把已下载好的文件删除重新下载
        LocalFileDownloadState::Incomplete(size) => start = size,
    }

    // 打开文件（续传时用 append + write）
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(args.save_absolute_path)
        .await
        .map_err(|e| e.to_string())?;

    let file_url = &args.resource_file_data.absolute_path;

    let thread_count =
        computed_semaphore_count(args.resource_file_data.size); // 计算线程数
    let semaphore = Arc::new(Semaphore::new(thread_count));

    let (tasks, mut download_task_option) =
        build_download_tasks(DownloadTaskOption {
            http_client: &args.http_client,
            file_url,
            semaphore,
            start,
            total_size,
            file,
        })
        .await?;

    // 等待所有任务完成
    let results = join_all(tasks).await;

    // 检查任务的结果
    for result in results {
        result
            .map_err(|e| format!("[chunked_download] {}", e.to_string()))?
            .map_err(|e| e.to_string())?;
    }

    download_task_option.file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}
