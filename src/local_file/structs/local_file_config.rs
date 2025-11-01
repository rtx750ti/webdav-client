use std::ops::Deref;
use crate::reactive::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct LocalFileConfigData {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片上传
    pub max_thread_count: u32,     // 最大线程数
    pub pause: bool,               // 暂停标志
}

type TLocalFileConfigData = ReactiveProperty<LocalFileConfigData>;

#[derive(Debug, Clone)]
pub struct LocalFileConfig {
    inner: TLocalFileConfigData,
}

impl LocalFileConfig {
    pub fn is_paused(&self) -> bool {
        self.get_current().map(|cfg| cfg.pause).unwrap_or(false)
    }
}

impl Deref for LocalFileConfig {
    type Target = TLocalFileConfigData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for LocalFileConfig {
    fn default() -> Self {
        Self {
            inner: ReactiveProperty::new(LocalFileConfigData {
                max_speed: None,
                timeout_secs: 0,
                max_retries: 0,
                large_file_threshold: 0,
                max_thread_count: 0,
                pause: false,
            }),
        }
    }
}

