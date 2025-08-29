use futures_util::future::{AbortHandle, AbortRegistration};
pub struct DownloadConfig {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub abort_handle: AbortHandle, // 停止信号
    _abort_reg: AbortRegistration, // 内部下载循环中使用，跟abort_handle配对使用
}

impl DownloadConfig {
    pub fn new(
        max_speed: Option<u64>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        Self {
            max_speed,
            timeout_secs,
            max_retries,
            abort_handle,
            _abort_reg: abort_reg,
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        Self {
            max_speed: None,  // 默认不限速
            timeout_secs: 30, // 默认超时 30 秒
            max_retries: 3,   // 默认最大重试 3 次
            abort_handle,
            _abort_reg: abort_reg,
        }
    }
}
