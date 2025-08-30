use crate::client::traits::folder::TResourcesFileCollectionList;
use crate::resources_file::structs::resources_file::ResourcesFile;

pub enum DownloadTask {
    ResourcesFileCollectionList(TResourcesFileCollectionList),
    ResourcesFile(ResourcesFile),
}

impl From<TResourcesFileCollectionList> for DownloadTask {
    fn from(value: TResourcesFileCollectionList) -> Self {
        DownloadTask::ResourcesFileCollectionList(value)
    }
}

impl From<ResourcesFile> for DownloadTask {
    fn from(value: ResourcesFile) -> Self {
        DownloadTask::ResourcesFile(value)
    }
}
