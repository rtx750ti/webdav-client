use crate::client::enums::{Depth, WebDavMethod};
use crate::client::structs::MultiStatus;
use crate::client::traits::remote::GetRemoteFoldersError;
use quick_xml::de::from_str;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:allprop/>
</D:propfind>"#;

pub(crate) async fn get_folders_raw_data(
    http_client: Client,
    absolute_url: &str,
    depth: &Depth,
) -> Result<MultiStatus, GetRemoteFoldersError> {
    // 组装请求头
    let mut headers = HeaderMap::new();
    headers
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    headers.insert("Depth", HeaderValue::from_static(depth.as_str()));
    headers.insert("Accept", HeaderValue::from_static("application/xml"));

    let method = WebDavMethod::PROPFIND
        .to_head_method()
        .map_err(|e| GetRemoteFoldersError::ToHeadMethodError(e))?;

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
        return Err(GetRemoteFoldersError::StatusParseError(format!(
            "状态解析异常 {status}: {xml}",
            status = status,
            xml = xml_text
        )));
    }

    let multi_status: MultiStatus = from_str(&xml_text)?;

    Ok(multi_status)
}
