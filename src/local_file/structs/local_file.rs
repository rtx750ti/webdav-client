use crate::local_file::structs::local_file_config::LocalFileConfig;
use crate::local_file::structs::local_file_data::LocalFileData;
use crate::local_file::structs::local_file_property::LocalFileProperty;
use reqwest::Client;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LocalFile {
    data: Arc<LocalFileData>,
    http_client: Client,
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
    pub async fn new(
        http_client: Client,
        absolute_path: &PathBuf,
    ) -> Result<Self, String> {
        let file_data = LocalFileData::new(absolute_path)
            .await
            .map_err(|e| e.to_string())?;

        // 这样写好debug，不然直接写到Ok里不好debug
        let self_struct = Self {
            data: Arc::new(file_data),
            http_client,
            reactive_state: LocalFileProperty {},
            reactive_config: LocalFileConfig {},
        };

        Ok(self_struct)
    }
}
