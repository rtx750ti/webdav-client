use crate::client::structs::client_key::ClientKey;

pub trait UrlFormat {
    /// 将用户输入的路径与 WebDAV 基础 URL 安全拼接，返回完整可访问的 URL 字符串。
    ///
    /// ## 安全检查
    /// 本方法会拒绝任何试图访问 `base_url` 之外资源的路径，包括但不限于：
    /// - 使用 `..` 回溯上级目录
    /// - 使用绝对路径跳转到不同目录（如 `/dav2/...`）
    /// - 使用完整 URL 指向不同 host / scheme
    /// - 使用 URL 编码绕过路径检查（如 `%2F`）
    ///
    /// ## 处理流程
    /// 1. 解析并验证 `base_url`（构造时已保证合法性）
    /// 2. 使用 [`Url::join`] 拼接 `path`，自动归一化路径
    /// 3. 校验拼接结果的 scheme、host、path 前缀是否与 `base_url` 一致
    ///
    /// ## 参数
    /// * `path` - 可以是相对路径、绝对路径或完整 URL，支持 `./`、URL 编码等
    ///
    /// ## 返回
    /// 成功时返回完整 URL（编码后的字符串），失败时返回 [`WebDavClientError`]
    ///
    /// ## 示例
    /// ```
    /// # use webdav_client::client::error::WebDavClientError;
    /// # use webdav_client::client::traits::url_trait::UrlParse;
    /// # use webdav_client::client::{WebDavClient, structs::webdav_child_client::WebDavChildClientKey};
    /// # async fn example() -> Result<(), WebDavClientError> {
    /// let client = WebDavClient::new();
    /// let key = WebDavChildClientKey::new(
    ///     "https://dav.example.com/dav/我的坚果云/",
    ///     "username"
    /// )?;
    ///
    /// let url_str = client.format_url_path(&key, "./书签").await?;
    /// assert_eq!(
    ///     url_str,
    ///     "https://dav.example.com/dav/%E6%88%91%E7%9A%84%E5%9D%9A%E6%9E%9C%E4%BA%91/%E4%B9%A6%E7%AD%BE"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn format_url_path(
        &self,
        web_dav_child_client_key: &ClientKey,
        path: &str,
    ) -> Result<String, String>;
}
