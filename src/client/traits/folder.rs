use crate::client::structs::client_key::ClientKey;
use crate::client::traits::account::AccountError;
use crate::public::enums::depth::Depth;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::to_resource_file_data::ToResourceFileDataError;
use async_trait::async_trait;
use crate::public::traits::url_format::UrlFormatError;

#[derive(Debug, thiserror::Error)]
pub enum GetFoldersError {
    #[error("HTTP 请求失败->{0}")]
    Http(#[from] reqwest::Error),

    #[error("XML 解析失败->{0}")]
    XmlParse(#[from] quick_xml::DeError),

    #[error("状态解析错误->{0}")]
    StatusParseError(String),

    #[error("资源文件出错->{0}")]
    ToResourceFileDataError(#[from] ToResourceFileDataError),

    #[error("URL 格式错误->{0}")]
    FormatUrlError(String),

    #[error("账号出错->{0}")]
    AccountError(#[from] AccountError),

    #[error("转换HeadMethod失败->{0}")]
    ToHeadMethodError(String),

    #[error("解析URL地址错误->{0}")]
    UrlFormatError(#[from] UrlFormatError)
}

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
