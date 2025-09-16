use crate::client::structs::client_key::TClientKey;
#[cfg(feature = "reactive")]
use crate::reactive::ReactiveProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceFileUniqueKey {
    pub client_key: TClientKey,
    pub relative_path: String,
}

#[cfg(not(feature = "reactive"))]
#[derive(Debug, Clone)]
pub struct ResourcesFile {
    data: Arc<ResourceFileData>,
    http_client: Client,
}

#[cfg(feature = "reactive")]
#[derive(Debug, Clone)]
pub struct ResourcesFile {
    /// 资源文件原始数据
    data: Arc<ResourceFileData>,
    http_client: Client,
    name: ReactiveProperty<String>,
    pause: ReactiveProperty<bool>,
    restart: ReactiveProperty<bool>,
    removed: ReactiveProperty<bool>,
}

impl ResourcesFile {
    #[cfg(not(feature = "reactive"))]
    pub fn new(data: ResourceFileData, http_client: Client) -> Self {
        Self { data: Arc::new(data), http_client }
    }

    #[cfg(feature = "reactive")]
    pub fn new(data: ResourceFileData, http_client: Client) -> Self {
        let name = ReactiveProperty::new(data.name.clone());
        let pause = ReactiveProperty::new(false);
        let restart = ReactiveProperty::new(false);
        let removed = ReactiveProperty::new(false);

        Self {
            data: Arc::new(data),
            http_client,
            name,
            pause,
            restart,
            removed,
        }
    }

    /// 获取资源文件的元数据
    pub fn get_data(&self) -> Arc<ResourceFileData> {
        self.data.clone()
    }

    /// 获取 HTTP 客户端
    pub fn get_http_client(&self) -> &Client {
        &self.http_client
    }
}

#[cfg(feature = "reactive")]
impl ResourcesFile {
    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }

    /// 获取暂停状态响应式属性
    pub fn get_reactive_pause(&self) -> &ReactiveProperty<bool> {
        &self.pause
    }

    /// 获取重启状态响应式属性
    pub fn get_reactive_restart(&self) -> &ReactiveProperty<bool> {
        &self.restart
    }

    /// 获取删除状态响应式属性
    pub fn get_reactive_removed(&self) -> &ReactiveProperty<bool> {
        &self.removed
    }

    /// 检查是否已删除
    pub fn is_removed(&self) -> Option<bool> {
        self.removed.get_current()
    }

    /// 标记为暂停
    pub fn pause(&self) -> Result<(), String> {
        self.pause.update(true)
    }

    /// 恢复
    pub fn resume(&self) -> Result<(), String> {
        self.pause.update(false)
    }

    /// 安全删除
    pub fn remove(&self) -> Result<(), String> {
        if let Some(is_removed) = self.is_removed() {
            if is_removed {
                return Err("资源已删除".into());
            }
        } else {
            return Err("删除失败，响应式结构已被销毁".to_string());
        }
        self.removed.update(true)
    }
}
