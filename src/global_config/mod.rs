#[cfg(feature = "reactive")]
use crate::reactive::ReactiveProperty;
#[cfg(feature = "reactive")]
use std::ops::Deref;

pub const DEFAULT_LARGE_FILE_THRESHOLD: u64 = 5 * 1024 * 1024;
pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024 * 1024; // 1GB 默认分片大小

#[cfg(not(feature = "reactive"))]
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    pub max_speed: Option<u64>,    // 限速
    pub timeout_secs: u64,         // 超时
    pub max_retries: u32,          // 最大重试次数
    pub large_file_threshold: u64, // 如果文件大于该值，则自动分片下载
    pub max_thread_count: u32,     // 最大线程数
    pub chunk_size: u64,           // 分片大小（字节）
    pub enable_chunked_upload: bool, // 启用分片上传
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
            chunk_size: DEFAULT_CHUNK_SIZE,
            enable_chunked_upload: true,
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
    pub enable_global_pause: bool, // 打开全局暂停功能
    pub global_pause: bool,        // 全局暂停标志
    pub chunk_size: u64,           // 分片大小（字节）
    pub enable_chunked_upload: bool, // 启用分片上传
}

#[cfg(feature = "reactive")]
impl Default for ConfigData {
    fn default() -> Self {
        Self {
            max_speed: None,
            timeout_secs: 30,
            max_retries: 4,
            large_file_threshold: DEFAULT_LARGE_FILE_THRESHOLD,
            enable_global_pause: false,
            global_pause: false,
            chunk_size: DEFAULT_CHUNK_SIZE,
            enable_chunked_upload: true,
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

    /// 设置分片大小（字节）
    ///
    /// # 参数
    /// * `size` - 分片大小，单位为字节。例如：1024*1024*1024 表示 1GB
    ///
    /// # 示例
    /// ```rust
    /// // 设置分片大小为 1GB
    /// config.set_chunk_size(1024 * 1024 * 1024)?;
    ///
    /// // 设置分片大小为 512MB
    /// config.set_chunk_size(512 * 1024 * 1024)?;
    /// ```
    pub fn set_chunk_size(&self, size: u64) -> Result<&Self, String> {
        if size == 0 {
            return Err("分片大小不能为0".to_string());
        }
        if size < 1024 * 1024 {
            return Err("分片大小不能小于1MB".to_string());
        }
        self.update_field(|cfg| cfg.chunk_size = size)?;
        Ok(self)
    }

    /// 启用分片上传
    pub fn enable_chunked_upload(&self) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.enable_chunked_upload = true)?;
        Ok(self)
    }

    /// 禁用分片上传
    pub fn disable_chunked_upload(&self) -> Result<&Self, String> {
        self.update_field(|cfg| cfg.enable_chunked_upload = false)?;
        Ok(self)
    }

    /// 获取当前分片大小
    pub fn get_chunk_size(&self) -> u64 {
        self.get_current()
            .map(|cfg| cfg.chunk_size)
            .unwrap_or(DEFAULT_CHUNK_SIZE)
    }

    /// 判断是否启用了分片上传
    pub fn is_chunked_upload_enabled(&self) -> bool {
        self.get_current()
            .map(|cfg| cfg.enable_chunked_upload)
            .unwrap_or(true)
    }

    /// 计算文件需要的分片数量
    ///
    /// # 参数
    /// * `file_size` - 文件大小（字节）
    ///
    /// # 返回
    /// 返回需要的分片数量
    ///
    /// # 示例
    /// ```rust
    /// // 2.5GB 文件，1GB 分片大小 = 3 个分片
    /// let chunks = config.calculate_chunk_count(2.5 * 1024.0 * 1024.0 * 1024.0 as u64);
    /// assert_eq!(chunks, 3);
    /// ```
    pub fn calculate_chunk_count(&self, file_size: u64) -> u64 {
        let chunk_size = self.get_chunk_size();
        (file_size + chunk_size - 1) / chunk_size
    }

    /// 便捷方法：直接设置分片大小（MB）
    ///
    /// # 参数
    /// * `size_mb` - 分片大小，单位为MB
    ///
    /// # 示例
    /// ```rust
    /// config.set_chunk_size_mb(512)?; // 设置为 512MB
    /// config.set_chunk_size_mb(1024)?; // 设置为 1GB
    /// ```
    pub fn set_chunk_size_mb(&self, size_mb: u64) -> Result<&Self, String> {
        self.set_chunk_size(size_mb * 1024 * 1024)
    }

    /// 便捷方法：直接设置分片大小（GB）
    ///
    /// # 参数
    /// * `size_gb` - 分片大小，单位为GB
    ///
    /// # 示例
    /// ```rust
    /// config.set_chunk_size_gb(1)?; // 设置为 1GB
    /// config.set_chunk_size_gb(2)?; // 设置为 2GB
    /// ```
    pub fn set_chunk_size_gb(&self, size_gb: u64) -> Result<&Self, String> {
        self.set_chunk_size(size_gb * 1024 * 1024 * 1024)
    }

    /// 获取分片大小（MB）
    pub fn get_chunk_size_mb(&self) -> f64 {
        self.get_chunk_size() as f64 / 1024.0 / 1024.0
    }

    /// 获取分片大小（GB）
    pub fn get_chunk_size_gb(&self) -> f64 {
        self.get_chunk_size() as f64 / 1024.0 / 1024.0 / 1024.0
    }

    /// 智能分片建议：根据文件大小推荐分片大小
    ///
    /// # 参数
    /// * `file_size` - 文件大小（字节）
    ///
    /// # 返回
    /// 推荐的分片大小（字节），最小为1MB
    pub fn suggest_chunk_size(&self, file_size: u64) -> u64 {
        if file_size < 100 * 1024 * 1024 {
            // 小于100MB：使用最小分片大小1MB（但通常建议简单上传）
            1024 * 1024 // 1MB
        } else if file_size < 1024 * 1024 * 1024 {
            // 100MB-1GB：256MB分片
            256 * 1024 * 1024
        } else if file_size < 10 * 1024 * 1024 * 1024 {
            // 1GB-10GB：512MB分片
            512 * 1024 * 1024
        } else {
            // 大于10GB：1GB分片
            1024 * 1024 * 1024
        }
    }

    /// 判断文件是否建议使用分片上传
    ///
    /// # 参数
    /// * `file_size` - 文件大小（字节）
    ///
    /// # 返回
    /// true 表示建议分片上传，false 表示建议简单上传
    pub fn should_use_chunked_upload(&self, file_size: u64) -> bool {
        file_size >= 100 * 1024 * 1024 // 100MB以上建议分片
    }

    /// 应用智能分片建议
    ///
    /// # 参数
    /// * `file_size` - 文件大小（字节）
    pub fn apply_smart_chunking(&self, file_size: u64) -> Result<&Self, String> {
        let suggested_size = self.suggest_chunk_size(file_size);
        self.set_chunk_size(suggested_size)
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
