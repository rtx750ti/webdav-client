use crate::client::structs::client_key::TClientKey;
use crate::global_config::global_config::GlobalConfig;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

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
    ) -> Result<(), String> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();

        let file_name_watcher = self.reactive_state.name.watch();
        let file_name = file_name_watcher.borrow();

        if force {
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_file_lock_value) = file_lock_option {
                file_lock.update_field(|file_lock| *file_lock = true)?;
                Ok(())
            } else {
                Err(format!("[lock_file] 文件[{:?}] 锁为空", file_name))
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
                                "[lock_file] 文件[{:?}] 尝试锁定{}次失败，因为已经被锁定",
                                file_name, FILE_LOCK_RETRY_TIMES
                            ));
                        }
                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;
                        continue;
                    } else {
                        // 文件未锁 → 立刻上锁
                        file_lock
                            .update_field(|file_lock| *file_lock = true)?;
                        return Ok(());
                    }
                }
                None => {
                    return Err(format!(
                        "[lock_file] 文件[{:?}] 锁为空",
                        file_name
                    ));
                }
            }
        }

        Err(format!("[lock_file] 文件[{:?}] 出现未知错误", file_name))
    }

    /// 解锁文件
    ///
    /// 传入true即可强制执行，但是需要自己承担风险
    pub(crate) async fn unlock_file(
        &self,
        force: bool,
    ) -> Result<(), String> {
        let file_lock = self.get_file_lock();
        let file_lock_watcher = file_lock.watch();
        let file_name_watcher = self.reactive_state.name.watch();
        let file_name = file_name_watcher.borrow();

        if force {
            // 不管状态如何，强制解锁
            let file_lock_option = file_lock_watcher.borrow();
            return if let Some(_) = file_lock_option {
                file_lock.update_field(|file_lock| *file_lock = false)?;
                Ok(())
            } else {
                Err(format!("[unlock_file] 文件[{:?}] 锁为空", file_name))
            };
        }

        for i in 0..FILE_LOCK_RETRY_TIMES {
            let file_lock_value = file_lock_watcher.borrow();
            match file_lock_value {
                Some(file_lock_value) => {
                    if !file_lock_value {
                        // 文件本来就没锁 → 无需解锁
                        if i == FILE_LOCK_RETRY_TIMES - 1 {
                            return Err(format!(
                                "[unlock_file] 文件[{:?}] 尝试解锁{}次失败",
                                file_name, FILE_LOCK_RETRY_TIMES
                            ));
                        }
                        tokio::time::sleep(FILE_LOCK_RETRY_DELAY).await;
                        continue;
                    } else {
                        // 文件已锁 → 解锁
                        file_lock.update_field(|file_lock| {
                            *file_lock = false
                        })?;
                        return Ok(());
                    }
                }
                None => {
                    return Err(format!(
                        "[unlock_file] 文件[{:?}] 锁为空",
                        file_name
                    ));
                }
            }
        }

        Err(format!("[unlock_file] 文件[{:?}] 出现未知错误", file_name))
    }
}
