use reqwest::Url;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
pub trait ToResourceFileData {
    fn to_resource_file_data(
        self,
        base_url: &Url,
    ) -> Result<Vec<ResourceFileData>, String>;
}
