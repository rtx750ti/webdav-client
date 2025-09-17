#[cfg(feature = "reactive")]
use crate::reactive::ReactiveProperty;
#[cfg(feature = "reactive")]
use std::ops::Deref;

pub const DEFAULT_LARGE_FILE_THRESHOLD: u64 = 20 * 1024 * 1024;

#[cfg(not(feature = "reactive"))]
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片下载
    pub max_thread_count: u32,     // 最大线程数
}

#[cfg(not(feature = "reactive"))]
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            max_speed: None,
            timeout_secs: 30,
            max_retries: 4,
            large_file_threshold: DEFAULT_LARGE_FILE_THRESHOLD,
            max_thread_count: 128,
        }
    }
}

#[cfg(feature = "reactive")]
#[derive(Debug, Clone)]
pub struct ConfigData {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片下载
    pub max_thread_count: u32,     // 最大线程数
    pub enable_global_pause: bool, // 打开全局暂停功能
    pub global_pause: bool,        // 全局暂停标志
}

#[cfg(feature = "reactive")]
impl Default for ConfigData {
    fn default() -> Self {
        Self {
            max_speed: None,
            timeout_secs: 30,
            max_retries: 4,
            large_file_threshold: DEFAULT_LARGE_FILE_THRESHOLD,
            max_thread_count: 128,
            enable_global_pause: false,
            global_pause: false,
        }
    }
}

/// # 全局配置
///
/// `GlobalConfig` 提供对配置数据的观察、更新和控制功能，
/// 并通过内部的 [`ReactiveProperty`] 实现响应式通知机制。
///
/// 该结构体可以安全地进行 `clone`，不会造成数据复制或状态丢失，
/// 因为内部使用了 [`Arc`] 进行共享引用管理，确保多处使用时仍保持一致性。
///
/// # 特性
/// - 支持线程安全的共享与更新
/// - 支持异步监听配置变化
/// - 支持链式调用语义方法（如暂停、恢复）
///
/// # 示例
/// ```rust
/// let config1 = GlobalConfig::default();
/// let config2 = config1.clone(); // ✅ 安全共享，无需担心状态丢失
///
/// config1.enable_pause_switch().unwrap();
/// assert!(config2.pause_enabled()); // 两者共享同一状态
/// ```
#[cfg(feature = "reactive")]
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    inner: ReactiveProperty<ConfigData>,
}

#[cfg(feature = "reactive")]
impl GlobalConfig {
    /// 创建新的全局配置
    ///
    /// 一般使用GlobalConfig::default()而不是new()
    pub fn new(config: ConfigData) -> Self {
        Self { inner: ReactiveProperty::new(config) }
    }

    /// 判断是否启用了全局暂停功能
    pub fn pause_enabled(&self) -> bool {
        self.get_current()
            .map(|cfg| cfg.enable_global_pause)
            .unwrap_or(false)
    }

    /// 判断当前是否处于暂停状态
    pub fn is_paused(&self) -> bool {
        self.get_current().map(|cfg| cfg.global_pause).unwrap_or(false)
    }

    /// 启用全局暂停功能（开关）
    pub fn enable_pause_switch(&self) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.enable_global_pause = true)?;
        Ok(self)
    }

    /// 禁用全局暂停功能（关闭开关）
    pub fn disable_pause_switch(&self) -> Result<&Self, String> {
        self.update_field(|cfg| {
            cfg.enable_global_pause = false;
            cfg.global_pause = false; // 同时取消暂停状态
        })?;
        Ok(self)
    }

    /// 尝试设置为暂停状态（仅当启用开关时才生效）
    pub fn try_pause(&self) -> Result<&Self, String> {
        if self.pause_enabled() {
            self.update_field(|cfg| cfg.global_pause = true)?;
            Ok(self)
        } else {
            Err("未启用全局暂停功能，无法暂停".to_string())
        }
    }

    /// 尝试恢复（仅当启用开关时才生效）
    pub fn try_resume(&self) -> Result<&Self, String> {
        if self.pause_enabled() {
            self.update_field(|cfg| cfg.global_pause = false)?;
            Ok(self)
        } else {
            Err("未启用全局暂停功能，无法恢复".to_string())
        }
    }

    /// 设置最大下载速度
    pub fn set_max_speed(
        &self,
        speed: Option<u64>,
    ) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.max_speed = speed)?;
        Ok(self)
    }

    /// 设置超时时间
    pub fn set_timeout(&self, seconds: u64) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.timeout_secs = seconds)?;
        Ok(self)
    }

    /// 设置最大重试次数
    pub fn set_max_retries(&self, retries: u32) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.max_retries = retries)?;
        Ok(self)
    }

    /// 设置大文件阈值
    pub fn set_large_file_threshold(
        &self,
        threshold: u64,
    ) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.large_file_threshold = threshold)?;
        Ok(self)
    }

    /// 设置最大线程数
    pub fn set_max_thread_count(
        &self,
        count: u32,
    ) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.max_thread_count = count)?;
        Ok(self)
    }
}

#[cfg(feature = "reactive")]
impl Default for GlobalConfig {
    fn default() -> Self {
        Self { inner: ReactiveProperty::new(ConfigData::default()) }
    }
}

#[cfg(feature = "reactive")]
impl Deref for GlobalConfig {
    type Target = ReactiveProperty<ConfigData>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
