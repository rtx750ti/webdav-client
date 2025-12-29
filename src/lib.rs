//! # webdav-client
//!
//! 一个基于 `reqwest` + `tokio` 的高性能 WebDAV 客户端库，提供：
//!
//! - 本地文件与远程文件的统一抽象
//! - WebDAV 客户端封装（GET、PUT、PROPFIND 等）
//! - 文件上传、下载、属性读取
//! - 响应式（Reactive）属性系统
//! - 全局配置管理
//!
//! 本库的设计目标是：**模块化、可扩展、易于集成**。
//!
//! ## 模块结构概览
//!
//! - [`client`] — WebDAV 客户端核心模块
//! - [`remote_file`] — 远程文件结构体、属性与下载功能
//! - [`local_file`] — 本地文件结构体、属性与上传功能
//! - [`reactive_property`] — 响应式属性系统
//! - [`global_config`] — 全局配置模块
//! - [`remote_lib`] — 对外暴露的第三方依赖（如 `dirs::home_dir`、`url::Url`）
//!
//! ## 设计理念
//!
//! - **隐藏内部实现**：所有内部模块均以 `_xxx` 命名并 `#[doc(hidden)]` 隐藏
//! - **对外暴露稳定 API**：通过 `pub mod xxx` 重新导出需要的类型与 trait
//! - **减少用户依赖负担**：将库内部使用的部分第三方依赖 re-export 出来


#[doc(hidden)]
mod _client;
#[doc(hidden)]
mod _global_config;
#[doc(hidden)]
mod _local_file;
#[doc(hidden)]
mod _reactive;
#[doc(hidden)]
mod _remote_file;

/// WebDAV 客户端模块。
///
/// 提供：
/// - `WebDavClient`：核心客户端
/// - `THttpClientArc`：线程安全的 HTTP 客户端包装
/// - WebDAV 方法枚举（如 `Depth`、`WebDavMethod`）
/// - URL 格式化工具
/// - 客户端相关的 key/value 结构体
/// - 客户端 trait（本地、远程、客户端自身）
///
/// 这是整个库的核心模块。
pub mod client {
    pub use crate::_client::{THttpClientArc, WebDavClient};

    /// WebDAV 相关枚举类型。
    pub mod enums {
        pub use crate::_client::enums::{depth::*, webdav_method::*};
    }

    /// 公共工具函数，例如 URL 处理。
    pub mod public {
        pub use crate::_client::public::format_base_url::*;
    }

    /// 客户端相关结构体。
    pub mod structs {
        pub use crate::_client::structs::{
            client_key::*, client_value::*, raw_file_xml::*,
            reactive_child_clients::*,
        };
    }

    /// 客户端相关 trait。
    pub mod traits {
        /// 本地文件相关 trait。
        pub mod local {
            pub use crate::_client::traits::local::*;
        }

        /// 远程文件相关 trait。
        pub mod remote {
            pub use crate::_client::traits::remote::folders::*;
        }

        /// 客户端自身 trait。
        pub mod client {
            pub use crate::_client::traits::_self::{
                account::*, url_format::*,
            };
        }
    }

    /// 客户端全局配置模块。
    pub mod global_config {
        pub use crate::_global_config::global_config::*;
    }
}

/// 响应式属性模块。
///
/// 提供一个轻量级的 reactive 系统，用于：
/// - 文件属性变更通知
/// - 客户端状态响应式更新
pub mod reactive_property {
    pub use crate::_reactive::reactive::*;
}

/// 远程文件模块。
///
/// 包含：
/// - 远程文件结构体（`RemoteFile`）
/// - 文件属性（`RemoteFileProperty`）
/// - 下载 trait
/// - 将 WebDAV XML 转换为结构体的工具
pub mod remote_file {
    /// 远程文件相关 trait。
    pub mod traits {
        /// 下载相关 trait。
        pub mod download {
            pub use crate::_remote_file::traits::download::*;
        }

        /// 将远程 XML 转换为结构体的 trait。
        pub mod to_remote_file_data {
            pub use crate::_remote_file::traits::to_remote_file_data::*;
        }
    }

    /// 远程文件相关结构体。
    pub mod structs {
        pub use crate::_remote_file::structs::{
            remote_file::*, remote_file_config::*, remote_file_data::*,
            remote_file_property::*,
        };
    }

    /// 远程文件 trait 的具体实现。
    pub mod impl_traits {
        pub use crate::_remote_file::impl_traits::impl_download::*;
    }
}

/// 本地文件模块。
///
/// 包含：
/// - 本地文件结构体（`LocalFile`）
/// - 本地文件属性
/// - 上传 trait
pub mod local_file {
    /// 本地文件相关 trait。
    pub mod traits {
        /// 上传相关 trait。
        pub mod upload {
            pub use crate::_local_file::traits::upload::*;
        }
    }

    /// 本地文件相关结构体。
    pub mod structs {
        pub use crate::_local_file::structs::{
            local_file::*, local_file_config::*, local_file_data::*,
            local_file_property::*,
        };
    }

    /// 本地文件 trait 的实现（当前为空）。
    pub mod impl_traits {}
}

/// 响应式系统的公共导出。
pub mod reactive {
    pub use crate::_reactive::reactive::*;
}

/// 全局配置模块。
///
/// 提供全局级别的配置管理，例如：
/// - 默认 WebDAV 设置
/// - 全局缓存
/// - 全局行为控制
pub mod global_config {
    pub use crate::_global_config::global_config::*;
}

/// 对外暴露的第三方依赖。
///
/// 这些依赖在库内部已经使用，因此直接 re-export，
/// 让使用者无需重复添加依赖。
///
/// 包含：
/// - `dirs::home_dir`
/// - `url::Url`
pub mod remote_lib {
    /// `dirs` 库的公共导出。
    pub mod dirs {
        pub use dirs::home_dir;
    }

    /// `url` 库的公共导出。
    pub mod url {
        pub use url::Url;
    }
}
