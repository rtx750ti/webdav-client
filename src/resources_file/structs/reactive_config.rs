use crate::reactive::ReactiveProperty;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ReactiveConfigData {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片下载
    pub max_thread_count: u32,     // 最大线程数
    pub global_pause: bool,        // 全局暂停标志
}

type TReactiveConfigData = ReactiveProperty<ReactiveConfigData>;

#[derive(Debug, Clone)]
pub struct ReactiveConfig {
    inner: TReactiveConfigData,
}

impl Deref for ReactiveConfig {
    type Target = TReactiveConfigData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for ReactiveConfig {
    fn default() -> Self {
        Self {
            inner: ReactiveProperty::new(ReactiveConfigData {
                max_speed: None,
                timeout_secs: 0,
                max_retries: 0,
                large_file_threshold: 0,
                max_thread_count: 0,
                global_pause: false,
            }),
        }
    }
}
