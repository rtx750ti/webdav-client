use crate::client::structs::client_key::ClientKey;
use crate::public::enums::depth::Depth;
use crate::resources_file::ResourcesFile;
use async_trait::async_trait;

#[async_trait]
pub trait Folders {
    async fn get_folders(
        &self,
        key: &ClientKey,
        reactive_paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<Vec<Vec<ResourcesFile>>, String>;
}
