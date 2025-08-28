use crate::client::WebDavClient;
use crate::client::structs::client_key::ClientKey;
use crate::client::structs::raw_file_xml::MultiStatus;
use crate::client::traits::account::Account;
use crate::client::traits::folder::Folders;
use crate::client::traits::to_resources_file::ToResourcesFile;
use crate::public::enums::depth::Depth;
use crate::public::enums::methods::WebDavMethod;
use crate::public::traits::url_format::UrlFormat;
use crate::resources_file::ResourcesFile;
use async_trait::async_trait;
use futures_util::future::join_all;
use quick_xml::de::from_str;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:allprop/>
</D:propfind>"#;

pub async fn get_folders_with_client(
    http_client: Client,
    absolute_url: &str,
    depth: &Depth,
) -> Result<MultiStatus, String> {
    // 组装请求头
    let mut headers = HeaderMap::new();
    headers
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    headers.insert("Depth", HeaderValue::from_static(depth.as_str()));
    headers.insert("Accept", HeaderValue::from_static("application/xml"));

    let method = WebDavMethod::PROPFIND.to_head_method()?;

    // 发送 PROPFIND 到基准目录（已保证有尾部斜杠）
    let res = http_client
        .request(method, absolute_url)
        .headers(headers)
        .body(PROPFIND_BODY)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();

    let xml_text = res.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() && status.as_u16() != 207 {
        return Err(format!(
            "状态解析异常 {status}: {xml}",
            status = status,
            xml = xml_text
        ));
    }

    let multi_status: MultiStatus =
        from_str(&xml_text).map_err(|e| e.to_string())?;

    Ok(multi_status)
}

#[async_trait]
impl Folders for WebDavClient {
    async fn get_folders(
        &self,
        key: &ClientKey,
        reactive_paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<Vec<Vec<ResourcesFile>>, String> {
        let http_client_arc = self.get_http_client(key)?;

        // 构建所有任务（这里只做并发请求）
        let tasks = reactive_paths.iter().map(|path| {
            let http_client_entity = http_client_arc.get_client();

            async move {
                let url = self.format_url_path(key, path)?;

                // 调用已有的单次请求函数
                get_folders_with_client(http_client_entity, &url, depth)
                    .await
            }
        });

        // 并发执行所有任务
        let results: Vec<Result<MultiStatus, String>> =
            join_all(tasks).await;

        // 这里只做简单收集，具体转换成 ResourcesFile 的逻辑你自己加
        let mut all_files = Vec::new();

        for res in results {
            match res {
                Ok(multi_status) => all_files.push(
                    multi_status
                        .to_resources_files(&key.get_base_url())?,
                ),
                Err(e) => {
                    eprintln!("请求失败: {}", e);
                }
            }
        }

        Ok(all_files)
    }
}
