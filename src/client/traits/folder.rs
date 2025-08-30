use crate::client::structs::client_key::ClientKey;
use crate::public::enums::depth::Depth;
use crate::public::utils::get_folders_public_impl::GetFoldersError;
use crate::resources_file::structs::resources_file::ResourcesFile;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum FoldersError {
    #[error("[get_folders] 获取文件夹函数出错->{0}")]
    GetFoldersError(#[from] GetFoldersError),
}

#[async_trait]
pub trait Folders {
    async fn get_folders(
        &self,
        key: &ClientKey,
        reactive_paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<Vec<Vec<ResourcesFile>>, GetFoldersError>;
}
