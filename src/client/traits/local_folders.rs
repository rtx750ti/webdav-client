use std::path::PathBuf;
use crate::client::structs::client_key::ClientKey;
use crate::local_file::structs::local_file::LocalFile;
use async_trait::async_trait;

/// 本地文件集合
pub type TLocalFileCollection = Vec<LocalFile>;

/// 文件构建错误信息
///
/// 当尝试构建 [`LocalFile`] 失败时，会生成此结构体记录错误原因和路径。
pub struct FileBuildError {
    /// 错误原因描述
    pub cause: String,
    /// 发生错误的文件路径
    pub path: PathBuf,
}

/// 文件构建失败列表
pub type TFileBuildFailedList = Vec<FileBuildError>;

/// 本地文件集合列表（包含多个本地文件集合）
pub type TLocalFileCollectionList = Vec<TLocalFileCollection>;

/// 本地文件夹结果
///
/// 返回一个元组：
/// - 第一个元素：成功构建的本地文件集合 [`TLocalFileCollection`]
/// - 第二个元素：构建失败的文件列表 [`TFileBuildFailedList`]
pub type LocalFoldersResult = (TLocalFileCollection, TFileBuildFailedList);

#[async_trait]
pub trait LocalFolders {
    /// 获取本地文件夹及其内容。
    ///
    /// # 参数
    ///
    /// * `key` - [`ClientKey`]，用于鉴权和标识客户端。
    /// * `paths` - **路径数组**，每个元素为一个本地文件系统路径字符串。
    ///   - 传入数组的方式可以一次性批量读取多个路径，提高处理效率。
    ///   - 支持文件路径和文件夹路径。
    ///
    /// # 返回值
    ///
    /// - 成功时返回 `Vec<Result<LocalFoldersResult, String>>`：
    ///   一个结果数组，其中每个元素对应一个输入路径的处理结果。
    ///   - `Ok(LocalFoldersResult)` 表示该路径处理成功，包含成功文件列表和失败文件列表。
    ///   - `Err(String)` 表示该路径处理失败，包含错误信息。
    /// - 失败时返回 `String` 错误信息（整体失败，如客户端未找到）。
    ///
    /// # ⚠️设计目的（必看1.1）
    ///
    /// 本方法专门用于 **批量读取本地文件系统**，避免多次调用带来的性能开销。
    /// 调用方应通过传入路径数组来批量获取多个文件或目录的内容，
    /// 从而获得更高的可控性与性能。
    ///
    /// # ⚠️ 注意（必看1.2）
    ///
    /// - **路径格式**：支持 **Windows** 和 **Unix/Linux** 风格的路径。
    /// - **文件与文件夹**：
    ///   - 如果路径是文件，则返回该文件的 [`LocalFile`]。
    ///   - 如果路径是文件夹，则返回该文件夹下所有文件的 [`LocalFile`] 列表（不递归子目录）。
    /// - **错误处理**：
    ///   - 如果某个文件构建失败，不会中断整个流程，而是记录到 [`TFileBuildFailedList`] 中。
    ///   - 如果读取目录时发生错误（如权限不足），会进行重试（最多3次，使用指数退避算法）。
    /// - **不存在的路径**：如果路径不存在，返回空的文件列表。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // ✅ 支持以下写法：
    /// let paths = vec![
    ///     "C:\\Users\\test\\file.txt".to_string(),      // Windows 文件路径
    ///     "/home/user/document.pdf".to_string(),        // Unix/Linux 文件路径
    ///     "C:\\Users\\test\\folder".to_string(),        // Windows 文件夹路径
    ///     "/home/user/downloads/".to_string(),          // Unix/Linux 文件夹路径
    /// ];
    ///
    /// let results = client.get_local_folders(&key, &paths).await?;
    ///
    /// for (i, result) in results.iter().enumerate() {
    ///     match result {
    ///         Ok((files, failed)) => {
    ///             println!("路径 {} 成功: {} 个文件, {} 个失败", i, files.len(), failed.len());
    ///         }
    ///         Err(e) => {
    ///             println!("路径 {} 失败: {}", i, e);
    ///         }
    ///     }
    /// }
    /// ```
    async fn get_local_folders(
        &self,
        key: &ClientKey,
        paths: &[String],
    ) -> Result<
        Vec<Result<LocalFoldersResult, String>>,
        String,
    >;
}
