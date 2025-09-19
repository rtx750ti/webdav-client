use crate::client::structs::client_key::TClientKey;
use crate::global_config::GlobalConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_config::ReactiveConfig;
#[cfg(feature = "reactive")]
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
#[cfg(feature = "reactive")]
use std::ops::Deref;
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
    reactive_state: ReactiveFileProperty,
    reactive_config: ReactiveConfig,
    global_config: GlobalConfig,
}

#[cfg(feature = "reactive")]
impl Deref for ResourcesFile {
    type Target = ReactiveFileProperty;

    fn deref(&self) -> &Self::Target {
        &self.reactive_state
    }
}

impl ResourcesFile {
    #[cfg(not(feature = "reactive"))]
    pub fn new(data: ResourceFileData, http_client: Client) -> Self {
        Self { data: Arc::new(data), http_client }
    }

    #[cfg(feature = "reactive")]
    pub fn new(
        data: ResourceFileData,
        http_client: Client,
        global_config: GlobalConfig,
    ) -> Self {
        let reactive_state = ReactiveFileProperty::new(data.name.clone());
        let reactive_config = ReactiveConfig::default();
        Self {
            data: Arc::new(data),
            http_client,
            reactive_state,
            reactive_config,
            global_config,
        }
    }

    pub fn get_reactive_state(&self) -> ReactiveFileProperty {
        self.reactive_state.clone()
    }

    pub fn get_reactive_config(&self) -> ReactiveConfig {
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
}
