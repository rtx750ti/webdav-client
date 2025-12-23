use crate::reactive::reactive::ReactiveProperty;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct LocalFileConfigData {
    /// 最大限速
    pub max_speed: Option<u64>,
    /// 超时
    pub timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 如果文件大于该值，则自动分片下载
    pub large_file_threshold: u64,
    /// 最大线程数
    pub max_thread_count: u32,
    /// 暂停标志
    pub pause: bool,
}

#[derive(Debug, Clone)]
pub struct LocalFileConfig {
    /// 配置内部数据
    inner: ReactiveProperty<LocalFileConfigData>,
}

impl LocalFileConfig {
    pub fn is_paused(&self) -> bool {
        self.get_current().map(|cfg| cfg.pause).unwrap_or(false)
    }
}

impl Deref for LocalFileConfig {
    type Target = ReactiveProperty<LocalFileConfigData>;

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
