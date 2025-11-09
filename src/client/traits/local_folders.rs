use crate::client::structs::client_key::ClientKey;
use crate::local_file::structs::local_file::LocalFile;
use async_trait::async_trait;
use tokio::fs::DirEntry;

pub type TLocalFileCollection = Vec<LocalFile>;

pub struct FileBuildError {
    pub cause: String,
    pub dir_entry: DirEntry,
}

pub type TFileBuildFailedList = Vec<FileBuildError>;

pub type TLocalFileCollectionList = Vec<TLocalFileCollection>;

pub type LocalFoldersResult = (TLocalFileCollection, TFileBuildFailedList);

#[async_trait]
pub trait LocalFolders {
    async fn get_local_folders(
        &self,
        key: &ClientKey,
        paths: &Vec<String>,
    ) -> Result<
        Vec<Result<LocalFoldersResult, String>>,
        String,
    >;
}
