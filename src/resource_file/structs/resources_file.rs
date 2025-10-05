use crate::client::structs::client_key::TClientKey;
use crate::global_config::global_config::GlobalConfig;
use crate::reactive::reactive::ReactivePropertyError;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use reqwest::Client;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceFileUniqueKey {
    pub client_key: TClientKey,
    pub relative_path: String,
}

// 锁的最大重试次数
const FILE_LOCK_RETRY_TIMES: usize = 3;

// 每次锁定/解锁的尝试间隔时间
const FILE_LOCK_RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct ResourcesFile {
    /// 资源文件原始数据
    data: Arc<ResourceFileData>,
    http_client: Client,
    reactive_state: ResourceFileProperty,
    reactive_config: ResourceConfig,
    global_config: GlobalConfig,
}

impl Deref for ResourcesFile {
    type Target = ResourceFileProperty;

    fn deref(&self) -> &Self::Target {
        &self.reactive_state
    }
}

#[derive(Debug, Error)]
pub enum LockFileError {
    /// 文件锁为空（未初始化或已销毁）
    #[error("文件[{0}] 锁为空")]
    LockIsNone(String),

    /// 文件已被锁定，重试后仍然失败
    #[error("文件[{0}] 已被锁定，尝试 {1} 次失败")]
    RetryLocked(String, usize),

    /// 设置锁定状态时失败（ReactiveProperty 内部错误）
    #[error("文件[{0}] 锁定失败: {1}")]
    SetLockedFailed(String, ReactivePropertyError),

    /// 未知错误
    #[error("文件[{0}] 出现未知错误")]
    Unknown(String),
}

#[derive(Debug, Error)]
pub enum UnlockFileError {
    /// 文件锁为空（未初始化或已销毁）
    #[error("文件[{0}] 锁为空")]
    LockIsNone(String),

    /// 文件未被锁定，重试后仍然失败
    #[error("文件[{0}] 未被锁定，尝试 {1} 次失败")]
    RetryUnlocked(String, usize),

    /// 设置解锁状态时失败（ReactiveProperty 内部错误）
    #[error("文件[{0}] 解锁失败: {1}")]
    SetUnlockedFailed(String, ReactivePropertyError),

    /// 未知错误
    #[error("文件[{0}] 出现未知错误")]
    Unknown(String),
}

impl ResourcesFile {
    pub fn new(
        data: ResourceFileData,
        http_client: Client,
        global_config: GlobalConfig,
    ) -> Self {
        let reactive_state = ResourceFileProperty::new(data.name.clone());
        let reactive_config = ResourceConfig::default();
        Self {
            data: Arc::new(data),
            http_client,
            reactive_state,
            reactive_config,
            global_config,
        }
    }

    pub fn get_reactive_state(&self) -> ResourceFileProperty {
        self.reactive_state.clone()
    }

    pub fn get_reactive_config(&self) -> ResourceConfig {
        self.reactive_config.clone()
    }

    /// 获取资源文件的原始数据
    pub fn get_data(&self) -> Arc<ResourceFileData> {
        self.data.clone()
    }

    /// 获取 HTTP 客户端
    pub fn get_http_client(&self) -> &Client {
        &self.http_client
    }

    pub fn get_global_config(&self) -> GlobalConfig {
        self.global_config.clone()
    }

    /// 锁定文件
    ///
    /// 传入true即可强制执行，但是需要自己承担风险
    pub(crate) async fn lock_file(
        &self,
        force: bool,
    ) -> Result<(), LockFileError> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();

        let file_name = self.data.name.clone();

        if force {
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_file_lock_value) = file_lock_option {
                file_lock
                    .update_field(|file_lock| *file_lock = true)
                    .map_err(|e| {
                        LockFileError::SetLockedFailed(file_name, e)
                    })?;

                Ok(())
            } else {
                Err(LockFileError::LockIsNone(file_name))
            };
        }

        for i in 0..FILE_LOCK_RETRY_TIMES {
            let file_lock_value = file_lock_watcher.borrow();
            match file_lock_value {
                Some(file_lock_value) => {
                    if file_lock_value {
                        // 文件已被锁
                        if i == FILE_LOCK_RETRY_TIMES - 1 {
                            return Err(LockFileError::RetryLocked(
                                file_name,
                                FILE_LOCK_RETRY_TIMES,
                            ));
                        }

                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;

                        continue;
                    } else {
                        // 文件未锁 → 立刻上锁
                        file_lock
                            .update_field(|file_lock| *file_lock = true)
                            .map_err(|e| {
                                LockFileError::SetLockedFailed(
                                    file_name, e,
                                )
                            })?;
                        return Ok(());
                    }
                }
                None => {
                    return Err(LockFileError::LockIsNone(file_name));
                }
            }
        }

        Err(LockFileError::Unknown(file_name))
    }

    /// 解锁文件
    ///
    /// 传入true即可强制执行，但是需要自己承担风险
    pub(crate) async fn unlock_file(
        &self,
        force: bool,
    ) -> Result<(), UnlockFileError> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();
        
        let file_name = self.data.name.clone();

        if force {
            // 不管状态如何，强制解锁
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_) = file_lock_option {
                file_lock
                    .update_field(|file_lock| *file_lock = false)
                    .map_err(|e| {
                        UnlockFileError::SetUnlockedFailed(file_name, e)
                    })?;

                Ok(())
            } else {
                Err(UnlockFileError::LockIsNone(file_name))
            };
        }

        for i in 0..FILE_LOCK_RETRY_TIMES {
            let file_lock_value = file_lock_watcher.borrow();
            match file_lock_value {
                Some(file_lock_value) => {
                    if !file_lock_value {
                        if i == FILE_LOCK_RETRY_TIMES - 1 {
                            return Err(UnlockFileError::RetryUnlocked(
                                file_name,
                                FILE_LOCK_RETRY_TIMES,
                            ));
                        }

                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;

                        continue;
                    } else {
                        // 文件已锁 → 解锁
                        file_lock
                            .update_field(|file_lock| *file_lock = false)
                            .map_err(|e| {
                                UnlockFileError::SetUnlockedFailed(
                                    file_name, e,
                                )
                            })?;

                        return Ok(());
                    }
                }
                None => {
                    return Err(UnlockFileError::LockIsNone(file_name));
                }
            }
        }

        Err(UnlockFileError::Unknown(file_name))
    }
}
