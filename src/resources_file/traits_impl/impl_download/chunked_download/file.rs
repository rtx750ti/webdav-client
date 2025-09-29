use crate::resources_file::traits_impl::impl_download::chunked_download::LocalFileDownloadState;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;

const DEFAULT_MAX_CONCURRENT_CHUNKS: u64 = 4; // 最大并发分片数

pub fn computed_semaphore_count(size: Option<u64>) -> usize {
    if let Some(size) = size {
        if size > 1000 * 1024 * 1024 {
            8
        } else if size > 750 * 1024 * 1024 {
            7
        } else if size > 500 * 1024 * 1024 {
            6
        } else if size > 250 * 1024 * 1024 {
            5
        } else if size > 100 * 1024 * 1024 {
            4
        } else if size > 50 * 1024 * 1024 {
            3
        } else if size > 25 * 1024 * 1024 {
            2
        } else {
            1
        }
    } else {
        DEFAULT_MAX_CONCURRENT_CHUNKS as usize
    }
}

pub async fn open_file(
    save_absolute_path: &PathBuf,
) -> Result<File, String> {
    // 打开文件（续传时用 append + write）
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(save_absolute_path)
        .await
        .map_err(|e| e.to_string())
}

/// 克隆一个文件句柄，用于并发访问同一个文件。
///
/// # 概述
/// 此函数会尝试创建一个新的 [`tokio::fs::File`] 句柄，
/// 它与原始句柄指向同一个底层文件（操作系统层面的文件描述符被复制）。
/// 克隆后的句柄可以在不同的异步任务中独立使用，
/// 特别适合 **分片下载** 这样的场景：
/// 多个任务需要同时写入同一个文件的不同区域。
///
/// # 为什么要克隆而不是共享？
/// - [`tokio::fs::File`] **不是 `Sync`**，不能安全地在多个异步任务中共享 `&mut File`。
/// - 如果使用 `Arc<Mutex<File>>`，虽然可以共享，但所有写操作会被串行化，
///   导致并发性能大幅下降。
/// - 通过克隆文件句柄，每个任务都能独立 `seek` 到自己的偏移量并写入数据，
///   避免了锁竞争，同时保持高效的并发写入。
///
/// # 错误
/// 如果底层操作系统在复制文件描述符时失败，
/// 函数会返回 `Err(String)`，其中包含原始 I/O 错误的描述。
///
/// # 示例
/// ```no_run
/// use tokio::fs::File;
///
/// # async fn example() -> Result<(), String> {
/// let file = File::create("example.txt").await.map_err(|e| e.to_string())?;
///
/// // 克隆文件句柄，供另一个任务独立写入
/// let cloned = clone_file_handle(&file).await?;
///
/// // 此时 `file` 和 `cloned` 都指向同一个物理文件
/// # Ok(())
/// # }
/// ```
///
/// # 相关链接
/// - [`tokio::fs::File::try_clone`]
/// - [`tokio::io::AsyncSeekExt`] 用于移动文件指针
/// - [`tokio::io::AsyncWriteExt`] 用于写入数据
pub async fn clone_file_handle(file: &File) -> Result<File, String> {
    file.try_clone().await.map_err(|e| format!("文件句柄克隆失败: {}", e))
}

pub async fn get_local_file_size(
    save_absolute_path: &PathBuf,
    total_size: u64,
) -> Result<LocalFileDownloadState, String> {
    // 检查是否已有部分文件（断点续传）
    let mut local_size: u64 = 0;

    let file_meta = tokio::fs::metadata(save_absolute_path).await;

    if let Ok(meta) = file_meta {
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
