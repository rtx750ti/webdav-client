use crate::client::WebDavClient;
use crate::client::structs::client_key::ClientKey;
use crate::public::traits::url_format::UrlFormat;

impl UrlFormat for WebDavClient {
    fn format_url_path(
        &self,
        key: &ClientKey,
        path: &str,
    ) -> Result<String, String> {
        let base_url = key.get_base_url();
        let joined_url = base_url.join(path).map_err(|e| e.to_string())?;

        let err = Err("路径越界，禁止访问上级目录".to_string());

        if !joined_url.as_str().starts_with(base_url.as_str()) {
            return err;
        }

        if joined_url.scheme() != base_url.scheme()
            || joined_url.host_str() != base_url.host_str()
            || !joined_url.path().starts_with(base_url.path())
        {
            return err;
        }

        Ok(joined_url.to_string())
    }
}
