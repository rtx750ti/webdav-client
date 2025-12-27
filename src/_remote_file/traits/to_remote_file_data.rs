use reqwest::Url;
use std::io;
use crate::remote_file::structs::RemoteFileData;

/// 资源文件的转换非常复杂，所以必须抽离成独立的错误
#[derive(Debug, thiserror::Error)]
pub enum ToRemoteFileDataError {
    #[error("资源文件数据转换失败: {0}")]
    ConversionFailed(String),

    #[error("无有效 PropStat 可用，可能没有 2xx 状态")]
    NoValidPropStat,

    #[error("URL 拼接失败: {0}")]
    UrlJoinError(String),

    #[error("ETag 解析错误: {0}")]
    ETagError(String),

    #[error("权限解析错误: {0}")]
    PrivilegesError(String),

    #[error("标准库错误: {0}")]
    Std(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoteFileDataError {
    #[error("转换资源文件错误: {0}")]
    ToRemoteFileData(#[from] ToRemoteFileDataError),
}

pub trait ToRemoteFileData {
    fn to_remote_file_data(
        self,
        base_url: &Url,
    ) -> Result<Vec<RemoteFileData>, ToRemoteFileDataError>;
}
