use async_trait::async_trait;
use crate::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;

#[async_trait]
impl Download for ResourcesFile{
    async fn download() -> Result<(), String> {
        Ok(())
    }
}