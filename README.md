# WebDAV Client

一个用 Rust 编写的高性能 WebDAV 客户端库，支持响应式模式（可选），并提供丰富的功能扩展。适合需要文件同步、下载管理、断点续传等场景的开发者。

---

## ✨ 核心特性

| 特性 | 说明 | 状态 |
|------|------|------|
| 🔄 **响应式支持** | 通过 `reactive` 特性开启，自动响应配置变化与事件流 | ✅ |
| 🚦 **限速控制** | 支持上传/下载速率限制 | ✅ |
| ⏸️ **暂停与恢复** | 任务可随时暂停、继续 | ✅ |
| 📂 **断点续传** | 下载/上传中断后可从中间继续 | ✅ |
| ⚡ **自定义并发** | 灵活控制并发数与线程池策略 | ✅ |
| 🔥 **热更新配置** | 无需重启即可更新配置 | ✅ |
| 📊 **进度监听** | 实时获取下载/上传进度 | ✅ |
| 🪝 **生命周期钩子** | 在任务开始、结束、错误时触发回调 | ✅ |
| 🧩 **策略自定义** | 冲突策略、并发策略等完全交给使用者实现 | ✅ |

---

## 📦 安装

> **注意**：暂时使用 GitHub 链接，后续会上传到 crates.io

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
webdav-client = { git = "https://github.com/rtx750ti/webdav-client", features = ["reactive"] }


## 🚀 快速开始

### 基础下载示例

```rust
use webdav_client::client::WebDavClient;
use webdav_client::client::enums::depth::Depth;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folders::Folders;
use webdav_client::resource_file::traits::download::Download;

#[tokio::test]
async fn test_download() -> Result<(), String> {
    let client = WebDavClient::new();

    // 添加账号
    let key = client
        .add_account(
            "https://dav.example.com",
            "username",
            "password",
        )
        .map_err(|e| e.to_string())?;

    // 获取文件夹内容
    let data = client
        .get_folders(&key, &vec!["./测试文件夹".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    // 下载所有文件
    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download("C:\\download\\")
                .await?;
        }
    }

    Ok(())
}
```

---

## ⚙️ 配置与扩展

### 1. 限速控制
可在创建客户端时配置速率限制。

### 2. 并发控制
支持自定义并发数与线程池。

### 3. 生命周期钩子
在任务开始、结束、失败时注册回调函数。

### 4. 下载进度监听
通过事件流获取实时进度。

### 5. 热更新
配置文件变更后可自动应用，无需重启。

---

## 🧭 设计理念

本库专注于**底层能力与灵活性**，而非强制策略。例如：

- **文件冲突处理**（覆盖、跳过、重命名） → 由开发者决定
- **并发策略**（FIFO、优先级队列等） → 由开发者实现

这样你可以根据业务需求自由扩展，而不是被框死在某种模式里。

---

## 📂 目录递归与响应式架构

为了实现真正的**全局响应式机制**，本库不提供默认的全局递归目录遍历。这是为了避免在响应式系统中引入不可控的深层状态依赖。

### 使用 `get_folders` 方法

如需递归获取目录内容，请使用 `get_folders` 方法：

| 特性 | 说明 |
|------|------|
| ✅ **混合路径** | 支持传入文件夹路径与文件路径混用 |
| ✅ **递归深度** | 支持指定递归深度（`Depth::One`、`Depth::Infinity`、`Depth::Zero`） |
| ✅ **结构化返回** | 返回结构化的资源列表，适用于后续下载、同步等操作 |

这种设计使得开发者可以根据业务需求灵活控制递归行为，同时保持响应式系统的稳定性与可预测性。

### 示例代码

```rust
use webdav_client::client::WebDavClient;
use webdav_client::client::enums::depth::Depth;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folders::Folders;

#[tokio::test]
async fn test_folders() -> Result<(), String> {
    let client = WebDavClient::new();

    let key = client
        .add_account(
            "https://dav.example.com",
            "username",
            "password",
        )
        .map_err(|e| e.to_string())?;

    // 支持混合传入文件夹和文件路径
    let _ = client
        .get_folders(
            &key,
            &vec![
                "./文件夹1/".to_string(),
                "./文件夹2/core.exe".to_string()
            ],
            &Depth::One
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

---

## 🌍 各语言 WebDAV 库对比

### 核心能力对比

| 语言 / 库 | 基础操作 | 许可证 |
|-----------|---------|--------|
| **Rust – 本库** | ✅ GET/PUT/DELETE/MKCOL | AGPL-3.0 |
| Java – Apache Jackrabbit | ✅ | Apache-2.0 |
| Python – PyWebDAV / pydav | ✅ | ZPL-2.0 |
| PHP – sabre/dav | ✅ | BSD |
| C – neon | ✅ | LGPL |
| C++ – libdavix | ✅ | LGPL |
| Go – golang.org/x/net/webdav | ✅ | BSD |
| JavaScript – webdav (npm) | ✅ | MIT |

### 高级特性对比

| 语言 / 库 | 断点续传 | 并发控制 | 响应式 | 下载进度 | 上传进度 | 生命周期钩子 | 热更新 |
|-----------|---------|---------|--------|---------|---------|------------|--------|
| **Rust – 本库** | ✅ | ✅ 自定义 | ✅ `reactive` | ✅ | ✅ | ✅ | ✅ |
| Java – Apache Jackrabbit | ❌ | 🟡 线程池 | ❌ | ❌ | ❌ | ❌ | ❌ |
| Python – PyWebDAV / pydav | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| PHP – sabre/dav | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| C – neon | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| C++ – libdavix | 🟡 部分 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Go – golang.org/x/net/webdav | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| JavaScript – webdav (npm) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

> **说明**：
> - 🟡 **部分支持**：可通过语言/框架的线程池或外部调度实现，库本身不内建策略
> - 本库的响应式机制通过 `reactive` 特性开启，关闭时即为普通 WebDAV 客户端
> - 断点续传、进度监听、钩子、热更新均为库的增强能力，便于构建下载器/同步器等复杂应用

---

## 🧪 性能测试：响应式属性修改

为了验证响应式属性的性能，下面是针对**单个**响应式属性进行的高频修改测试：

### 测试参数

| 项目 | 数值 |
|------|------|
| **总修改次数** | 1,000,000 次 |
| **监听器数量** | 30 个 |
| **每次修改数据大小** | 255 字节（随机字符串） |
| **总耗时** | 2.17 秒 |
| **平均每次修改耗时** | ≈ 2.17 微秒 |
| **物理内存使用** | 5,922,816 bytes ≈ 5.65 MB |
| **虚拟内存使用** | 1,064,960 bytes ≈ 1.02 MB |

### 测试环境

- **Runtime**: 单线程 Tokio runtime
- **配置**: 默认配置，无限速，无 IO 操作
- **CPU**: Intel Core i5-14600K
- **内存**: 64GB DDR5 5600MHz
- **GPU**: AMD Radeon RX 7650 GRE
- **测试代码**: `tests/traits_impl_test/download.rs::test_reactive_data()`

### 🔥 性能亮点

- ✅ **高吞吐量**：百万级别操作仅需 2.17 秒，平均每次修改 2.17 微秒
- ✅ **低内存占用**：物理内存仅 5.65 MB，适合嵌入式或资源受限场景
- ✅ **高并发支持**：30 个监听器同时工作，无明显性能瓶颈
- ✅ **稳定性强**：长时间高频操作下内存占用稳定

---

## 📌 TODO / 未来计划

- [ ] 更丰富的错误类型与诊断信息
- [ ] 上传任务支持
- [ ] 更完善的文档与示例
- [ ] 发布到 crates.io

---

## 📚 参考资料

- [WebDAV 软件比较 - 维基百科](https://zh.wikipedia.org/wiki/WebDAV%E8%BD%AF%E4%BB%B6%E6%AF%94%E8%BE%83)
- [Awesome WebDAV (GitHub)](https://github.com/fstanis/awesome-webdav)

---

## 📄 许可证

本项目采用 **AGPL-3.0** 许可证。详见 [LICENSE](LICENSE) 文件。

