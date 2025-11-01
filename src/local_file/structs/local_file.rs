use crate::local_file::structs::local_file_config::LocalFileConfig;
use crate::local_file::structs::local_file_data::LocalFileData;
use crate::local_file::structs::local_file_property::LocalFileProperty;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

// 锁的最大等待时间（秒）
const FILE_LOCK_TIMEOUT_SECS: u64 = 3;

/// 本地文件领域对象
///
/// `LocalFile` 是一个响应式的本地文件管理对象，提供文件操作的响应式接口。
///
/// # 特性
///
/// - **响应式**: 所有状态变化都是响应式的，可以通过 `watch()` 监听变化
/// - **线程安全**: 内部使用 `Arc` 包装，可以安全地在多线程环境中使用
/// - **文件锁**: 提供文件锁机制，防止并发操作冲突
///
/// # 示例
///
/// ```rust,no_run
/// use webdav_client::local_file::structs::local_file::LocalFile;
///
/// #[tokio::main]
/// async fn main() -> Result<(), String> {
///     // 创建本地文件对象
///     let local_file = LocalFile::new("/path/to/file.txt").await?;
///
///     // 获取响应式状态
///     let state = local_file.get_reactive_state();
///
///     // 监听文件名变化
///     let name_watcher = state.get_reactive_name().watch();
///     println!("文件名: {:?}", name_watcher.borrow());
///
///     // 监听上传字节数变化
///     let bytes_watcher = state.get_upload_bytes().watch();
///     println!("已上传字节: {:?}", bytes_watcher.borrow());
///
///     Ok(())
/// }
/// ```
///
/// # 注意
///
/// - 文件路径必须是绝对路径
/// - 如果文件不存在，会自动创建
/// - 所有错误都返回 `String` 类型（功能稳定后会改为专门的错误类型）
#[derive(Debug, Clone)]
pub struct LocalFile {
    /// 本地文件原始数据
    data: Arc<LocalFileData>,
    reactive_state: LocalFileProperty,
    reactive_config: LocalFileConfig,
}

impl Deref for LocalFile {
    type Target = LocalFileProperty;

    fn deref(&self) -> &Self::Target {
        &self.reactive_state
    }
}

impl LocalFile {
    /// 创建一个新的本地文件对象
    ///
    /// # 参数
    ///
    /// * `absolute_path` - 文件的绝对路径
    ///
    /// # 返回值
    ///
    /// - 成功时返回 `LocalFile` 对象
    /// - 失败时返回错误信息字符串
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use webdav_client::local_file::structs::local_file::LocalFile;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), String> {
    ///     let file = LocalFile::new("/path/to/file.txt").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(absolute_path: &str) -> Result<Self, String> {
        let data = LocalFileData::new(&absolute_path.into()).await?;
        let meta = data.get_meta().await?;

        let reactive_state = LocalFileProperty::new(meta.name.clone());
        let reactive_config = LocalFileConfig::default();

        Ok(Self {
            data: Arc::new(data),
            reactive_state,
            reactive_config,
        })
    }

    /// 获取响应式状态对象
    ///
    /// 返回包含文件名、上传字节数、文件锁等响应式属性的对象
    pub fn get_reactive_state(&self) -> LocalFileProperty {
        self.reactive_state.clone()
    }

    /// 获取响应式配置对象
    ///
    /// 返回包含限速、超时、重试次数等配置的响应式对象
    pub fn get_reactive_config(&self) -> LocalFileConfig {
        self.reactive_config.clone()
    }

    /// 获取本地文件的原始数据
    ///
    /// 返回包含文件句柄和路径信息的原始数据对象
    pub fn get_data(&self) -> Arc<LocalFileData> {
        self.data.clone()
    }

    /// 判断是否为目录
    ///
    /// # 返回值
    ///
    /// - `true` - 如果是目录
    /// - `false` - 如果是文件
    pub fn is_dir(&self) -> bool {
        matches!(self.data.as_ref(), LocalFileData::Directory { .. })
    }

    /// 判断是否为文件
    ///
    /// # 返回值
    ///
    /// - `true` - 如果是文件
    /// - `false` - 如果是目录
    pub fn is_file(&self) -> bool {
        matches!(self.data.as_ref(), LocalFileData::File { .. })
    }

    /// 锁定文件
    ///
    /// # 参数
    /// * `force` - 是否强制锁定（即使已被锁定也会成功）
    ///
    /// # 返回值
    /// - `Ok(())` - 锁定成功
    /// - `Err(String)` - 锁定失败（超时或其他错误）
    ///
    /// # 注意
    /// - 如果 `force=false`，会尝试在超时时间内获取锁
    /// - 如果 `force=true`，会立即获取锁（即使已被其他线程锁定）
    pub(crate) async fn lock_file(&self, force: bool) -> Result<(), String> {
        let file_lock_mutex = self.get_file_lock_mutex();

        // 获取文件名用于错误消息（使用缓存的名称）
        let file_name = self.get_reactive_name()
            .get_current()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if force {
            // 强制模式：直接获取锁并设置为 true
            let mut lock_guard = file_lock_mutex.lock().await;
            *lock_guard = true;

            // 更新响应式状态
            self.update_lock_state(true)?;

            return Ok(());
        }

        // 非强制模式：尝试在超时时间内获取锁
        let timeout = Duration::from_secs(FILE_LOCK_TIMEOUT_SECS);

        match tokio::time::timeout(timeout, async {
            let mut lock_guard = file_lock_mutex.lock().await;

            if *lock_guard {
                // 已被锁定
                Err(format!("文件[{}] 已被锁定", file_name))
            } else {
                // 未被锁定，立即上锁
                *lock_guard = true;

                // 更新响应式状态
                self.update_lock_state(true)?;

                Ok(())
            }
        }).await {
            Ok(result) => result,
            Err(_) => Err(format!(
                "文件[{}] 锁定超时（{}秒）",
                file_name, FILE_LOCK_TIMEOUT_SECS
            )),
        }
    }

    /// 解锁文件
    ///
    /// # 参数
    /// * `force` - 是否强制解锁（即使未被锁定也会成功）
    ///
    /// # 返回值
    /// - `Ok(())` - 解锁成功
    /// - `Err(String)` - 解锁失败（文件未被锁定或其他错误）
    ///
    /// # 注意
    /// - 如果 `force=false`，只有在文件已被锁定时才会解锁
    /// - 如果 `force=true`，无论当前状态如何都会设置为未锁定
    pub(crate) async fn unlock_file(&self, force: bool) -> Result<(), String> {
        let file_lock_mutex = self.get_file_lock_mutex();

        // 获取文件名用于错误消息（使用缓存的名称）
        let file_name = self.get_reactive_name()
            .get_current()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if force {
            // 强制模式：直接获取锁并设置为 false
            let mut lock_guard = file_lock_mutex.lock().await;
            *lock_guard = false;

            // 更新响应式状态
            self.update_lock_state(false)?;

            return Ok(());
        }

        // 非强制模式：检查是否已锁定，只有已锁定才解锁
        let mut lock_guard = file_lock_mutex.lock().await;

        if !*lock_guard {
            // 未被锁定，返回错误
            Err(format!("文件[{}] 未被锁定，无法解锁", file_name))
        } else {
            // 已被锁定，执行解锁
            *lock_guard = false;

            // 更新响应式状态
            self.update_lock_state(false)?;

            Ok(())
        }
    }
}
