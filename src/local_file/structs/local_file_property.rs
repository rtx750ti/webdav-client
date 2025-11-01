use crate::reactive::reactive::ReactiveProperty;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct LocalFileProperty {
    pub name: ReactiveProperty<String>,
    pub upload_bytes: ReactiveProperty<usize>,
    /// 文件锁，使用 Mutex 确保原子性操作
    /// 内部的 bool 值表示是否被锁定，Mutex 本身提供互斥访问
    file_lock: Arc<Mutex<bool>>,
    /// 文件锁状态的响应式属性，仅用于观察，不用于控制
    file_lock_state: ReactiveProperty<bool>,
}

impl LocalFileProperty {
    pub fn new(name: String) -> Self {
        Self {
            name: ReactiveProperty::new(name),
            upload_bytes: ReactiveProperty::new(0),
            file_lock: Arc::new(Mutex::new(false)),
            file_lock_state: ReactiveProperty::new(false),
        }
    }

    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }

    pub fn get_upload_bytes(&self) -> &ReactiveProperty<usize> {
        &self.upload_bytes
    }

    /// 获取文件锁的 Mutex（用于实际的锁控制）
    pub(crate) fn get_file_lock_mutex(&self) -> Arc<Mutex<bool>> {
        self.file_lock.clone()
    }

    /// 获取文件锁状态的响应式属性（仅用于观察）
    pub fn get_file_lock_state(&self) -> &ReactiveProperty<bool> {
        &self.file_lock_state
    }

    /// 更新文件锁状态的响应式属性（内部使用）
    pub(crate) fn update_lock_state(&self, locked: bool) -> Result<(), String> {
        self.file_lock_state
            .update(locked)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

