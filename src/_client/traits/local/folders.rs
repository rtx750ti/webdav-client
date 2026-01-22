use crate::client::enums::Depth;
use crate::client::structs::ClientKey;
use crate::local_file::structs::LocalFile;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
#[derive(Debug, Clone)]
pub struct LocalFiles {
    pub files: Vec<LocalFile>,
}

impl IntoIterator for LocalFiles {
    type Item = LocalFile;
    type IntoIter = std::vec::IntoIter<LocalFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl LocalFiles {
    pub fn new() -> Self {
        Self { files: Default::default() }
    }
}

impl Deref for LocalFiles {
    type Target = Vec<LocalFile>;

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

pub type FromPath = String;
pub type LocalFileCollectionsProperty = HashMap<FromPath, LocalFiles>;

#[derive(Debug, Clone)]
pub struct LocalFileCollections {
    pub file_collections: LocalFileCollectionsProperty,
}

impl LocalFileCollections {
    pub fn new() -> Self {
        Self { file_collections: Default::default() }
    }
}

impl IntoIterator for LocalFileCollections {
    type Item = (FromPath, LocalFiles);
    type IntoIter = std::collections::hash_map::IntoIter<
        crate::client::traits::remote::FromPath,
        LocalFiles,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.file_collections.into_iter()
    }
}

impl Deref for LocalFileCollections {
    type Target = LocalFileCollectionsProperty;

    fn deref(&self) -> &Self::Target {
        &self.file_collections
    }
}

impl DerefMut for LocalFileCollections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file_collections
    }
}

#[async_trait]
pub trait LocalFolders {
    async fn get_local_folders(
        &self,
        key: &ClientKey,
        paths: &[&str],
    ) -> Result<LocalFileCollections, String>;
}
