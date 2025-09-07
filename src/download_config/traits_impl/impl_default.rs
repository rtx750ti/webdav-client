use futures_util::future::AbortHandle;
use crate::download_config::DownloadConfig;

impl Default for DownloadConfig {
    fn default() -> Self {
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        Self {
            auto_download_folder: false,
            max_speed: None,  // 默认不限速
            timeout_secs: 30, // 默认超时 30 秒
            max_retries: 4,   // 默认最大重试 4 次
            abort_handle,
            abort_reg,
        }
    }
}
