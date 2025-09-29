use crate::client::structs::raw_file_xml::MultiStatus;
use crate::client::traits::account::AccountError;
use crate::public::enums::depth::Depth;
use crate::public::enums::methods::WebDavMethod;
use crate::public::traits::url_format::UrlFormatError;
use crate::resources_file::traits::to_resource_file_data::ToResourceFileDataError;
use quick_xml::de::from_str;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;

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
    UrlFormatError(#[from] UrlFormatError),

    #[error("无法找到对应的资源收集器->账号:{0}地址:{1}")]
    NotFindResourceCollector(String, String),
}

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:allprop/>
</D:propfind>"#;

pub async fn get_folders_with_client(
    http_client: Client,
    absolute_url: &str,
    depth: &Depth,
) -> Result<MultiStatus, GetFoldersError> {
    // 组装请求头
    let mut headers = HeaderMap::new();
    headers
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    headers.insert("Depth", HeaderValue::from_static(depth.as_str()));
    headers.insert("Accept", HeaderValue::from_static("application/xml"));

    let method = WebDavMethod::PROPFIND
        .to_head_method()
        .map_err(|e| GetFoldersError::ToHeadMethodError(e))?;

    // 发送 PROPFIND 到基准目录（已保证有尾部斜杠）
    let res = http_client
        .request(method, absolute_url)
        .headers(headers)
        .body(PROPFIND_BODY)
        .send()
        .await?;

    let status = res.status();

    let xml_text = res.text().await?;

    if !status.is_success() && status.as_u16() != 207 {
        return Err(GetFoldersError::StatusParseError(format!(
            "状态解析异常 {status}: {xml}",
            status = status,
            xml = xml_text
        )));
    }

    let multi_status: MultiStatus = from_str(&xml_text)?;

    Ok(multi_status)
}
