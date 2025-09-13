use crate::client::THttpClientArc;
use crate::client::structs::client_key::ClientKey;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum AddAccountError {
    #[error("创建Key错误->{0}")]
    CreateKeyError(String),
    #[error("创建HTTP客户端错误->{0}")]
    CreateHttpClientError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveAccountError {
    // 客户端未释放
    #[error("客户端未释放->{0}")]
    ClientInUse(String),
    // 客户端删除失败
    #[error("客户端删除失败->{0}")]
    DeleteFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveAccountForceError {
    #[error("强制删除失败->{0}")]
    RemoveError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GetHttpClientError {
    #[error("查找失败->{0}")]
    NotFindClient(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("[add_account] 新增账号函数出错->{0}")]
    AddAccountError(#[from] AddAccountError),
    #[error("[remove_account] 删除账号函数出错->{0}")]
    RemoveAccountError(#[from] RemoveAccountError),
    #[error("[get_http_client] 强制删除账号函数出错->{0}")]
    RemoveAccountForceError(#[from] RemoveAccountForceError),
    #[error("[remove_account_force] 获取HTTP客户端函数出错->{0}")]
    GetHttpClientError(#[from] GetHttpClientError),
}

/// 定义账户管理功能的通用接口，
/// 用于添加、移除和获取 HTTP 客户端实例。
pub trait Account {
    /// 添加一个新账户，并返回它在客户端池中的唯一键。
    ///
    /// # 参数
    /// * `base_url` - WebDAV 服务的基础 URL，例如 `https://example.com/dav`
    /// * `username` - 登录用户名
    /// * `password` - 登录密码
    ///
    /// # 返回值
    /// 成功时返回 `ClientKey`（客户端唯一标识）；
    /// 失败时返回 `String` 错误信息。
    ///
    /// # 实现细节
    /// 通常会在这里创建一个 `Arc` 包裹的 HTTP 客户端实例，
    /// 并存入 `HashMap<ClientKey, Arc<THttpClientArc>>` 里进行管理。
    fn add_account(
        &self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<ClientKey, AccountError>;

    /// 移除指定账户的客户端实例。
    ///
    /// # 参数
    /// * `key` - 对应的客户端唯一标识。
    ///
    /// # 返回值
    /// 成功返回 `()`，失败时返回错误信息字符串。
    ///
    /// # 注意
    /// 调用前可配合 [`can_modify_value`] 检查当前实例是否安全移除。
    fn remove_account(
        &self,
        key: &ClientKey,
    ) -> Result<(), AccountError>;

    /// 根据客户端键获取 HTTP 客户端实例。
    ///
    /// # 参数
    /// * `key` - 对应的客户端唯一标识。
    ///
    /// # 返回值
    /// 成功时返回 `Arc` 克隆出的 `THttpClientArc`，调用方获得独立的强引用；
    /// 失败时返回错误信息字符串。
    ///
    /// # 实现建议
    /// 推荐用 `.cloned()` 获取 `Arc` 的新副本，确保调用方持有自己的引用，
    /// 不依赖 `HashMap` 的生命周期。
    fn get_http_client(
        &self,
        key: &ClientKey,
    ) -> Result<THttpClientArc, AccountError>;

    /// **强制删除**指定账户的客户端实例，不做任何引用计数检查。
    ///
    /// ⚠️ **危险操作（DANGEROUS OPERATION）**
    ///
    /// 该方法会直接从内部存储（例如 `HashMap<ClientKey, Arc<THttpClientArc>>`）
    /// 中移除对应的客户端实例，而 **不检查** 是否仍有其他地方持有该实例的强引用。
    ///
    /// 如果外部代码仍然持有 `Arc<THttpClientArc>`，此操作不会立即导致悬垂引用，
    /// 但会让管理容器失去对该客户端的跟踪，可能引发：
    /// - 资源生命周期失控（连接无法正常释放或复用）
    /// - 外部逻辑访问已被业务层视为“删除”的账户
    /// - 状态不一致或不可预期的副作用
    ///
    /// # 参数
    /// * `key` - 要删除的客户端唯一标识。
    ///
    /// # 返回值
    /// 成功返回 `()`，失败返回错误信息字符串。
    ///
    /// # 使用建议
    /// - **仅在**你百分之百确定此客户端实例不再需要，
    ///   或需要紧急释放资源而不关心外部引用时使用。
    /// - 常规删除应优先使用 [`remove_account`] 配合 [`can_modify_value`] 检查安全性。
    ///
    /// # 示例
    /// ```ignore
    /// // 强制移除，忽略引用计数
    /// account.remove_account_force(&client_key)?;
    /// ```
    fn remove_account_force(
        &self,
        key: &ClientKey,
    ) -> Result<(), AccountError>;
}
