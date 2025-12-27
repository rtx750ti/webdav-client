use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// 对应 WebDAV 响应 XML 顶层的 `<D:multistatus>` 节点
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MultiStatus {
    /// `<D:response>` 节点列表，每个 response 表示一个资源（文件或目录）
    #[serde(rename = "response", default)]
    pub responses: Vec<Response>,
}

/// 对应单个 `<D:response>` 节点
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Response {
    /// `<D:href>`：资源路径（URL 编码，需要解码才能显示原始文件名）
    pub href: String,
    /// `<D:propstat>`：资源属性集和对应状态码的列表
    #[serde(rename = "propstat", default)]
    pub propstats: Vec<PropStat>,
}

/// 对应 `<D:propstat>` 节点：一个属性集 + 对应的 HTTP 状态
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PropStat {
    /// `<D:prop>`：资源的具体属性
    pub prop: Prop,
    /// `<D:status>`：该属性集对应的 HTTP 状态，如 "HTTP/1.1 200 OK"
    pub status: String,
}

/// 对应 `<D:prop>` 节点，列出资源的所有属性
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Prop {
    /// `<resourcetype>`：资源类型（文件/目录）
    #[serde(rename = "resourcetype")]
    pub resource_type: Option<ResourceType>,

    /// `<getcontentlength>`：文件大小（字节），目录一般没有此字段
    #[serde(rename = "getcontentlength")]
    pub content_length: Option<u64>,

    /// `<getlastmodified>`：最后修改时间（HTTP-date 格式）
    #[serde(
        rename = "getlastmodified",
        deserialize_with = "de_http_date",
        default
    )]
    pub last_modified: Option<DateTime<FixedOffset>>,

    /// `<getcontenttype>`：MIME 类型（如 "text/plain" 或 "application/pdf"）
    #[serde(rename = "getcontenttype")]
    pub content_type: Option<String>,

    /// `<creationdate>`：资源创建时间（ISO8601，通常以 Z 结尾表示 UTC）
    #[serde(rename = "creationdate")]
    pub creation_date: Option<String>,

    /// `<getetag>`：实体标签（文件内容的标识符，可用于缓存或变更检测）
    #[serde(rename = "getetag")]
    pub etag: Option<String>,

    /// `<displayname>`：显示名（用户友好的文件/目录名）
    #[serde(rename = "displayname")]
    pub display_name: Option<String>,

    /// `<owner>`：资源所有者（例如邮箱账号）
    pub owner: Option<String>,

    /// `<current-user-privilege-set>`：当前用户对该资源的权限集合
    #[serde(rename = "current-user-privilege-set")]
    pub current_user_privilege_set: Option<CurrentUserPrivilegeSet>,
}

/// 将 HTTP-date 格式的时间解析为 `DateTime<FixedOffset>`
fn de_http_date<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        DateTime::parse_from_rfc2822(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

/// `<resourcetype>` 节点
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ResourceType {
    /// `<collection/>` 存在表示是目录，否则是文件
    #[serde(rename = "collection")]
    pub is_collection: Option<EmptyElement>,
}

/// 空元素的占位结构，例如 `<collection/>`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmptyElement {}

/// `<current-user-privilege-set>` 节点
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CurrentUserPrivilegeSet {
    /// 多个 `<privilege>` 子节点
    #[serde(rename = "privilege", default)]
    pub privileges: Vec<Privilege>,
}

/// `<privilege>` 节点，可包含 read/write/all 等权限
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Privilege {
    pub read: Option<EmptyElement>,
    pub write: Option<EmptyElement>,
    pub all: Option<EmptyElement>,
    pub read_acl: Option<EmptyElement>,
    pub write_acl: Option<EmptyElement>,
}
