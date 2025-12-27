use crate::reactive::reactive::ReactiveProperty;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct RemoteFileConfigData {
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

type TRemoteConfigData = ReactiveProperty<RemoteFileConfigData>;

#[derive(Debug, Clone)]
pub struct RemoteConfig {
    /// 配置内部数据
    inner: ReactiveProperty<RemoteFileConfigData>,
}

impl RemoteConfig {
    pub fn is_paused(&self) -> bool {
        self.get_current().map(|cfg| cfg.pause).unwrap_or(false)
    }
}

impl Deref for RemoteConfig {
    type Target = TRemoteConfigData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            inner: ReactiveProperty::new(RemoteFileConfigData {
                max_speed: None,
                timeout_secs: 15,
                max_retries: 10,
                large_file_threshold: 100 * 1024, // 100mb
                max_thread_count: 6,
                pause: false,
            }),
        }
    }
}
