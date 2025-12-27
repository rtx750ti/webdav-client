use crate::local_file::structs::LocalFile;
use crate::local_file::traits::upload::Upload;
use async_trait::async_trait;

#[async_trait]
impl Upload for LocalFile {
    async fn upload(&self, remote_path: &str) -> Result<(), String> {
       Ok(())
    }
}
