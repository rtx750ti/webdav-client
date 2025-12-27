mod _client;
mod _local_file;
mod _remote_file;
mod global_config;
mod reactive;

/// 客户端模块
pub mod client {
    pub use crate::_client::{THttpClientArc, WebDavClient};
    pub mod enums {
        pub use crate::_client::enums::{depth::*, webdav_method::*};
    }

    pub mod public {
        pub use crate::_client::public::format_base_url as format_webdav_base_url;
    }

    pub mod structs {
        pub use crate::_client::structs::{
            client_key::*, client_value::*, raw_file_xml::*,
            reactive_child_clients::*,
        };
    }

    pub mod traits {
        pub mod local {
            pub use crate::_client::traits::local::*;
        }

        pub mod remote {
            pub use crate::_client::traits::remote::folders::*;
        }

        pub mod client {
            pub use crate::_client::traits::_self::{
                account::*, url_format::*,
            };
        }
    }

    pub mod global_config {
        pub use crate::global_config::global_config::*;
    }
}

pub mod reactive_property {
    pub use crate::reactive::reactive::*;
}
pub mod remote_file {
    pub mod traits {
        pub mod download {
            pub use crate::_remote_file::traits::download::*;
        }

        pub mod to_remote_file_data {
            pub use crate::_remote_file::traits::to_remote_file_data::*;
        }
    }

    pub mod structs {
        pub use crate::_remote_file::structs::{
            remote_file::*, remote_file_config::*, remote_file_data::*,
            remote_file_property::*,
        };
    }

    pub mod impl_traits {
        pub use crate::_remote_file::impl_traits::impl_download::*;
    }
}

pub mod local_file {
    pub mod traits {
        pub mod upload {
            pub use crate::_local_file::traits::upload::*;
        }
    }
    pub mod structs {
        pub use crate::_local_file::structs::{
            local_file::*, local_file_config::*, local_file_data::*,
            local_file_property::*,
        };
    }
    pub mod impl_traits {}
}
