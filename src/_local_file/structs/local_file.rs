use crate::_global_config::global_config::GlobalConfig;
use crate::_local_file::structs::local_file_config::LocalFileConfig;
use crate::_local_file::structs::local_file_data::LocalFileData;
use crate::_local_file::structs::local_file_property::LocalFileProperty;
use reqwest::Client;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct LocalFile {
    data: Arc<LocalFileData>,
    http_client: Client,
    reactive_state: LocalFileProperty,
    reactive_config: LocalFileConfig,
    global_config: GlobalConfig,
}

impl fmt::Debug for LocalFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalFile")
            .field(
                "http_client",
                &"<HttpClient with hidden authorization>",
            )
            .field("data", &self.data)
            .field("reactive_state", &self.reactive_state)
            .field("reactive_config", &self.reactive_config)
            .field("global_config", &self.global_config)
            .finish()
    }
}

impl LocalFile {
    pub fn new(
        http_client: Client,
        local_path: &str,
        global_config: GlobalConfig,
    ) -> Result<Self, String> {
        let local_file_data = LocalFileData::new(local_path)?;
        let local_file_data = Arc::new(local_file_data);

        let reactive_state = LocalFileProperty::new();
        let reactive_config = LocalFileConfig::default();

        Ok(Self {
            http_client,
            data: local_file_data,
            reactive_state,
            reactive_config,
            global_config,
        })
    }

    pub fn get_data(&self) -> Arc<LocalFileData> {
        self.data.clone()
    }

    pub fn get_reactive_state(&self) -> LocalFileProperty {
        self.reactive_state.clone()
    }
}
