pub mod enums;
pub mod traits;
mod traits_impl;

#[derive(Clone)]
pub struct DownloadConfig {
    pub auto_download_folder: bool, // 自动下载文件夹
    pub max_speed: Option<u64>,     // 限速
    pub timeout_secs: u64,          // 超时
    pub max_retries: u32,           // 最大重试次数
    pub large_file_threshold: u64,  // 如果文件大于该值，则自动分片下载
    pub max_thread_count: u32,      // 最大线程数
}

impl DownloadConfig {
    pub fn new(
        auto_download_folder: bool,
        max_speed: Option<u64>,
        timeout_secs: u64,
        max_retries: u32,
        large_file_threshold: u64,
        max_thread_count: u32,
    ) -> Self {
        Self {
            auto_download_folder,
            max_speed,
            timeout_secs,
            max_retries,
            large_file_threshold,
            max_thread_count,
        }
    }
}
