use crate::download_config::DownloadConfig;
use futures_util::future::AbortHandle;

/// 如果文件大小大于这个值，则自动分片下载
const DEFAULT_LARGE_FILE_THRESHOLD: u64 = 20 * 1024 * 1024;

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            auto_download_folder: false,
            max_speed: None,  // 默认不限速
            timeout_secs: 30, // 默认超时 30 秒
            max_retries: 4,   // 默认最大重试 4 次
            large_file_threshold: DEFAULT_LARGE_FILE_THRESHOLD,
            max_thread_count: 128, // 最大128线程
        }
    }
}
