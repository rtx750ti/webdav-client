use chrono::{DateTime, FixedOffset};

pub mod error;
pub mod structs;
pub mod traits;
pub mod traits_impl;

#[derive(Debug, Clone)]
pub struct ResourcesFile {
    pub relative_root_path: String, // 文件的相对路径（相对根目录）
    pub absolute_path: String,      // 文件的完整路径（从 href 拿到）
    pub name: String,               // 友好化的文件或目录名
    pub is_dir: bool,               // 是否目录
    pub size: Option<u64>,          // 文件大小（字节）
    pub last_modified: Option<DateTime<FixedOffset>>, // 原始时间
    pub mime: Option<String>,       // MIME 类型
    pub owner: Option<String>,      // 所有者
    pub etag: Option<String>,       // 清理后的 ETag
    pub privileges: Vec<String>,    // 权限列表
}
