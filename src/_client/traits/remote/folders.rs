use crate::client::enums::Depth;
use crate::client::structs::ClientKey;
use crate::client::traits::client::{AccountError, UrlFormatError};
use crate::remote_file::structs::RemoteFile;
use crate::remote_file::traits::to_remote_file_data::ToRemoteFileDataError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, thiserror::Error)]
pub enum GetRemoteFoldersError {
    #[error("HTTP 请求失败->{0}")]
    Http(#[from] reqwest::Error),

    #[error("XML 解析失败->{0}")]
    XmlParse(#[from] quick_xml::DeError),

    #[error("状态解析错误->{0}")]
    StatusParseError(String),

    #[error("资源文件出错->{0}")]
    ToRemoteFileDataError(#[from] ToRemoteFileDataError),

    #[error("URL 格式错误->{0}")]
    FormatUrlError(String),

    #[error("账号出错->{0}")]
    AccountError(#[from] AccountError),

    #[error("转换HeadMethod失败->{0}")]
    ToHeadMethodError(String),

    #[error("解析URL地址错误->{0}")]
    UrlFormatError(#[from] UrlFormatError),
}

#[derive(Debug, Clone)]
pub struct RemoteFiles {
    pub files: Vec<RemoteFile>,
}

impl IntoIterator for RemoteFiles {
    type Item = RemoteFile;
    type IntoIter = std::vec::IntoIter<RemoteFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl RemoteFiles {
    pub fn new() -> Self {
        Self { files: Default::default() }
    }
}

impl Deref for RemoteFiles {
    type Target = Vec<RemoteFile>;

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

impl DerefMut for RemoteFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.files
    }
}

pub type FromPath = String;
pub type RemoteFileCollectionsProperty = HashMap<FromPath, RemoteFiles>;

#[derive(Debug, Clone)]
pub struct RemoteFileCollections {
    pub file_collections: RemoteFileCollectionsProperty,
}

impl RemoteFileCollections {
    pub fn new() -> Self {
        Self { file_collections: Default::default() }
    }
}

impl IntoIterator for RemoteFileCollections {
    type Item = (FromPath, RemoteFiles);
    type IntoIter =
        std::collections::hash_map::IntoIter<FromPath, RemoteFiles>;

    fn into_iter(self) -> Self::IntoIter {
        self.file_collections.into_iter()
    }
}

impl Deref for RemoteFileCollections {
    type Target = RemoteFileCollectionsProperty;

    fn deref(&self) -> &Self::Target {
        &self.file_collections
    }
}

impl DerefMut for RemoteFileCollections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file_collections
    }
}

#[async_trait]
pub trait RemoteFolders {
    async fn get_remote_folders(
        &self,
        key: &ClientKey,
        paths: &[&str],
        depth: &Depth,
    ) -> Result<RemoteFileCollections, GetRemoteFoldersError>;
}
