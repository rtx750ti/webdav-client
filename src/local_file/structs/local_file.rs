use crate::local_file::structs::local_file_config::LocalFileConfig;
use crate::local_file::structs::local_file_data::LocalFileData;
use crate::local_file::structs::local_file_property::LocalFileProperty;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

// 锁的最大重试次数
const FILE_LOCK_RETRY_TIMES: usize = 3;

// 每次锁定/解锁的尝试间隔时间
const FILE_LOCK_RETRY_DELAY: Duration = Duration::from_secs(1);

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
    /// 传入true即可强制执行，但是需要自己承担风险
    pub(crate) async fn lock_file(&self, force: bool) -> Result<(), String> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();

        let file_name = self
            .data
            .get_meta()
            .await
            .map(|m| m.name)
            .unwrap_or_else(|_| "unknown".to_string());

        if force {
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_file_lock_value) = file_lock_option {
                file_lock
                    .update_field(|file_lock| *file_lock = true)
                    .map_err(|e| format!("文件[{}] 锁定失败: {}", file_name, e))?;

                Ok(())
            } else {
                Err(format!("文件[{}] 锁为空", file_name))
            };
        }

        for i in 0..FILE_LOCK_RETRY_TIMES {
            let file_lock_value = file_lock_watcher.borrow();
            match file_lock_value {
                Some(file_lock_value) => {
                    if file_lock_value {
                        // 文件已被锁
                        if i == FILE_LOCK_RETRY_TIMES - 1 {
                            return Err(format!(
                                "文件[{}] 已被锁定，尝试 {} 次失败",
                                file_name, FILE_LOCK_RETRY_TIMES
                            ));
                        }

                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;

                        continue;
                    } else {
                        // 文件未锁 → 立刻上锁
                        file_lock
                            .update_field(|file_lock| *file_lock = true)
                            .map_err(|e| {
                                format!("文件[{}] 锁定失败: {}", file_name, e)
                            })?;
                        return Ok(());
                    }
                }
                None => {
                    return Err(format!("文件[{}] 锁为空", file_name));
                }
            }
        }

        Err(format!("文件[{}] 出现未知错误", file_name))
    }

    /// 解锁文件
    ///
    /// 传入true即可强制执行，但是需要自己承担风险
    pub(crate) async fn unlock_file(&self, force: bool) -> Result<(), String> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();

        let file_name = self
            .data
            .get_meta()
            .await
            .map(|m| m.name)
            .unwrap_or_else(|_| "unknown".to_string());

        if force {
            // 不管状态如何，强制解锁
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_) = file_lock_option {
                file_lock
                    .update_field(|file_lock| *file_lock = false)
                    .map_err(|e| format!("文件[{}] 解锁失败: {}", file_name, e))?;

                Ok(())
            } else {
                Err(format!("文件[{}] 锁为空", file_name))
            };
        }

        for i in 0..FILE_LOCK_RETRY_TIMES {
            let file_lock_value = file_lock_watcher.borrow();
            match file_lock_value {
                Some(file_lock_value) => {
                    if !file_lock_value {
                        if i == FILE_LOCK_RETRY_TIMES - 1 {
                            return Err(format!(
                                "文件[{}] 未被锁定，尝试 {} 次失败",
                                file_name, FILE_LOCK_RETRY_TIMES
                            ));
                        }

                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;

                        continue;
                    } else {
                        // 文件已锁 → 解锁
                        file_lock
                            .update_field(|file_lock| *file_lock = false)
                            .map_err(|e| {
                                format!("文件[{}] 解锁失败: {}", file_name, e)
                            })?;

                        return Ok(());
                    }
                }
                None => {
                    return Err(format!("文件[{}] 锁为空", file_name));
                }
            }
        }

        Err(format!("文件[{}] 出现未知错误", file_name))
    }
}
