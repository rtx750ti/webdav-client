use crate::client::enums::depth::Depth;
use crate::client::structs::client_key::ClientKey;
use crate::client::webdav_request::get_folders_with_client::GetFoldersError;
use crate::resource_file::structs::resources_file::ResourcesFile;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum FoldersError {
    #[error("[get_folders] 获取文件夹函数出错->{0}")]
    GetFoldersError(#[from] GetFoldersError),
}

/// 资源文件集合
pub type TResourcesFileCollection = Vec<ResourcesFile>;
/// 资源文件组（包含多个资源文件集合）
pub type TResourcesFileCollectionList = Vec<TResourcesFileCollection>;

#[async_trait]
pub trait Folders {
    /// 获取远程文件夹及其内容。
    ///
    /// # 参数
    ///
    /// * `key` - [`ClientKey`]，用于鉴权和标识客户端。
    /// * `paths` - **路径数组**，每个元素为一个 Linux 标准路径字符串（如 `/foo/bar`）。
    ///   - 传入数组的方式可以一次性批量请求多个路径，避免递归调用带来的性能负担。  
    /// * `depth` - [`Depth`] 枚举，指定查询深度（例如 `Depth::One` 仅获取一层，`Depth::Infinity` 获取所有子目录）。
    ///
    /// # 返回值
    ///
    /// - 成功时返回 [`TResourcesFileCollectionList`]：  
    ///   一个“资源文件集合列表”，其中每个集合对应一个输入路径的结果。
    /// - 失败时返回 [`GetFoldersError`]，调用方可根据错误类型进行处理。
    ///
    /// # ⚠️设计目的（必看1.1）
    ///
    /// 本方法专门用于 **替代递归调用**，以去除递归带来的隐式调度问题。
    /// 调用方应通过传入路径数组来批量获取多个目录的内容，
    /// 而不是在库内部依赖递归逻辑，从而获得更高的可控性与性能。
    ///
    /// # ⚠️ 注意（必看1.2）
    ///
    /// - **路径格式**：仅支持 **Linux 标准路径字符串**（`/` 分隔），不支持 Windows 风格路径。  
    /// - **末尾斜杠**：路径末尾可以带 `/` 也可以不带 `/`，两种写法均被支持。  
    /// - **相对路径**：支持 `./` 开头的相对路径。  
    /// - **安全限制**：  
    ///   1. 禁止访问上一级目录（`..`），调用方不能跳出最初设定的 WebDAV 根目录。  
    ///   2. 禁止直接访问根目录 `/`，调用方必须指定具体的子路径。  
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // ✅ 支持以下写法：
    /// let paths = vec![
    ///     "/test1-folder/test1.doc".to_string(),   // 文件路径（绝对路径）
    ///     "./test2.pdf".to_string(),               // 文件路径（相对路径）
    ///     "/test2-folders/".to_string(),           // 文件夹路径（末尾带 /）
    ///     "./test2-folders/".to_string(),          // 相对路径写法
    ///     "/test3-folder".to_string(),             // 文件夹路径（末尾不带 /）
    /// ];
    ///
    /// ❌ 不允许：
    /// "../" 或 "/../" 这样的路径，不能跳出 WebDAV 根目录⚠️
    /// "/" 根目录路径，禁止直接访问⚠️
    async fn get_folders(
        &self,
        key: &ClientKey,
        paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<TResourcesFileCollectionList, GetFoldersError>;
}
