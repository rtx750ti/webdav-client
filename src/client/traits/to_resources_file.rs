use crate::resources_file::ResourcesFile;
use async_trait::async_trait;
use reqwest::Url;

pub trait ToResourcesFile {
    fn to_resources_files(
        self,
        base_url: &Url,
    ) -> Result<Vec<ResourcesFile>, String>;
}
