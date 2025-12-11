use crate::local_file::structs::local_file_config::LocalFileConfig;
use crate::local_file::structs::local_file_data::LocalFileData;
use crate::local_file::structs::local_file_property::LocalFileProperty;
use reqwest::Client;
use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use crate::global_config::global_config::GlobalConfig;

#[derive(Clone)]
pub struct LocalFile {
    /// 文件数据
    data: Arc<LocalFileData>,
    /// http客户端
    http_client: Client,
    /// 文件状态（响应式）
    reactive_state: LocalFileProperty,
    /// 文件配置（响应式）
    reactive_config: LocalFileConfig,
    /// 全局配置（响应式）
    global_config: GlobalConfig,
}

impl fmt::Debug for LocalFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalFile")
            .field("data", &self.data)
            .field("http_client", &"<Client with hidden authorization>")
            .field("reactive_state", &self.reactive_state)
            .field("reactive_config", &self.reactive_config)
            .finish()
    }
}

impl Deref for LocalFile {
    type Target = LocalFileProperty;

    fn deref(&self) -> &Self::Target {
        &self.reactive_state
    }
}

impl LocalFile {
    pub async fn new(
        http_client: Client,
        absolute_path: &PathBuf,
        global_config: GlobalConfig,
    ) -> Result<Self, String> {
        let file_data = LocalFileData::new(absolute_path)
            .await
            .map_err(|e| e.to_string())?;

        let reactive_state = LocalFileProperty::new();

        let reactive_config = LocalFileConfig::default();

        // 这样写好debug，不然直接写到Ok里不好debug
        let self_struct = Self {
            data: Arc::new(file_data),
            http_client,
            reactive_state,
            reactive_config,
            global_config
        };

        Ok(self_struct)
    }

    pub fn get_reactive_state(&self) -> LocalFileProperty {
        self.reactive_state.clone()
    }

    pub fn get_reactive_config(&self) -> LocalFileConfig {
        self.reactive_config.clone()
    }

    /// 获取资源文件的原始数据
    pub fn get_data(&self) -> Arc<LocalFileData> {
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
