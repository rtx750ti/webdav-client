use crate::client::structs::client_key::ClientKey;
use crate::client::webdav_request::get_folders_public_impl::GetFoldersError;
use crate::resources_file::structs::resources_file::ResourcesFile;
use async_trait::async_trait;
use crate::client::enums::depth::Depth;

#[derive(Debug, thiserror::Error)]
pub enum FoldersError {
    #[error("[get_folders] 获取文件夹函数出错->{0}")]
    GetFoldersError(#[from] GetFoldersError),
}

/// 资源文件集合
pub type TResourcesFileCollection = Vec<ResourcesFile>;
/// 资源文件组（包含多个资源文件集合）
pub type TResourcesFileCollectionList = Vec<TResourcesFileCollection>;

#[async_trait]
pub trait Folders {
    async fn get_folders(
        &self,
        key: &ClientKey,
        reactive_paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<TResourcesFileCollectionList, GetFoldersError>;
}
