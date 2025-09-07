pub mod enums;
pub mod traits;
mod traits_impl;

use futures_util::future::{AbortHandle, AbortRegistration};

pub struct DownloadConfig {
    pub auto_download_folder: bool, // 自动下载文件夹
    pub max_speed: Option<u64>,     // 限速
    pub timeout_secs: u64,          // 超时
    pub max_retries: u32,           // 最大重试次数
    pub abort_handle: AbortHandle,  // 停止信号
    pub abort_reg: AbortRegistration, // 内部下载循环中使用，跟abort_handle配对使用
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片下载
}

impl DownloadConfig {
    pub fn new(
        auto_download_folder: bool,
        max_speed: Option<u64>,
        timeout_secs: u64,
        max_retries: u32,
        large_file_threshold: u64,
    ) -> Self {
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        Self {
            auto_download_folder,
            max_speed,
            timeout_secs,
            max_retries,
            abort_handle,
            abort_reg,
            large_file_threshold,
        }
    }
}
