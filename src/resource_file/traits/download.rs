use crate::global_config::global_config::GlobalConfig;
use crate::reactive::reactive::ReactivePropertyError;
use crate::resource_file::impl_traits::impl_download::handle_download::HandleDownloadError;
use crate::resource_file::impl_traits::impl_download::{
    HandleMountedError, HandleUnmountedError, PreprocessingSavePathError,
};
use crate::resource_file::structs::resources_file::{
    LockFileError, UnlockFileError,
};
use async_trait::async_trait;
use std::sync::Arc;

/// 下载过程中可能出现的错误类型。
///
/// 该枚举将下载流程中涉及的各类错误统一封装，
/// 方便调用方进行错误处理和分类。
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    /// 保存路径预处理失败，例如路径不存在或权限不足。
    #[error(transparent)]
    PreprocessingSavePathError(#[from] PreprocessingSavePathError),

    /// 处理已挂载资源时出错。
    #[error(transparent)]
    HandleMountedError(#[from] HandleMountedError),

    /// 处理未挂载资源时出错。
    #[error(transparent)]
    HandleUnmountedError(#[from] HandleUnmountedError),

    /// 下载过程中发生错误，例如网络请求失败或写入失败。
    #[error(transparent)]
    HandleDownloadError(#[from] HandleDownloadError),

    /// 文件加锁失败，可能是并发访问冲突。
    #[error(transparent)]
    LockFileError(#[from] LockFileError),

    /// 文件解锁失败。
    #[error(transparent)]
    UnlockFileError(#[from] UnlockFileError),
}

/// 下载配置类型别名。
///
/// 当前直接使用 [`GlobalConfig`] 作为下载配置，
/// 未来如需扩展可在此处替换为更具体的配置结构。
///
/// - 别名都以`T`开头命名，如"TDownloadConfig"
pub type TDownloadConfig = GlobalConfig;

/// 定义下载行为的异步 trait。
///
/// 实现该 trait 的类型需要提供一个 `download` 方法，
/// 用于将资源下载到指定的输出路径。
///
/// # 返回值
///
/// - 成功时返回 [`Arc<Self>`]，方便在多线程环境下共享下载后的对象。
/// - 失败时返回 [`DownloadError`]，调用方可根据错误类型决定后续处理逻辑。
#[async_trait]
pub trait Download {
    /// 执行下载操作。
    ///
    /// # 参数
    ///
    /// * `output_absolute_path` - 下载文件的输出路径（绝对路径）。
    ///
    /// # 返回值
    ///
    /// - 成功时返回 [`Arc<Self>`]，方便链式调用。
    /// - 失败时返回 [`DownloadError`]，调用方可根据错误类型决定后续处理逻辑。
    ///
    /// # 错误
    ///
    /// 如果下载过程中出现任何问题，将返回 [`DownloadError`]。
    ///
    /// # ⚠️注意（一定要看）
    ///
    /// 1、本方法 **不包含任何重试机制**，出现错误即返回错误。
    ///
    /// 如果需要重试，请使用者自行实现（例如在外层包裹重试逻辑）。
    ///
    /// 2、本方法**不包含递归逻辑**，仅下载一层目录。
    ///
    /// ️ 若需要递归请自行实现，实现请参考WebdavClient中的get_folders方法⚠️。
    async fn download(
        self,
        output_absolute_path: &str,
    ) -> Result<Arc<Self>, DownloadError>;
}
