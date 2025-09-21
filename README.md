webdav-client是一个用 Rust 编写的高性能 WebDAV 客户端库，支持响应式模式（可选），并提供丰富的功能扩展。 适合需要
文件同步、下载管理、断点续传 等场景的开发者。

## ✨ 特性

🔄 响应式支持：通过 reactive 特性开启，自动响应配置变化与事件流。✅

🚦 限速控制：支持上传/下载速率限制。✅

⏸ 暂停与恢复：任务可随时暂停、继续。✅

📂 断点续传：下载/上传中断后可从中间继续。✅

⚡ 自定义并发：灵活控制并发数与线程池策略。✅

🔥 热更新配置：无需重启即可更新配置。✅

📊 进度监听：实时获取下载进度。✅

🪝 生命周期钩子：在任务开始、结束、错误时触发回调。✅

🧩 策略交由开发者决定：冲突策略、并发策略等不做强制约束，完全交给使用者实现。✅

## 📦 安装（暂时使用github链接，后续会上传crates.io）
在 Cargo.toml 中添加依赖：

toml
[dependencies]
webdav-client = { git = "https://github.com/rtx750ti/webdav-client", features = ["reactive"] }

如果不需要响应式支持，可以去掉 features = ["reactive"]。

---

🚀 快速开始

下面是一个简单的下载示例：

``` rust
#[tokio::test]
async fn test_download() -> Result<(), String> {
let client = WebDavClient::new();
let webdav_account = load_account(WEBDAV_ENV_PATH_2);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    let data = client
        .get_folders(&key, &vec!["./测试文件夹".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                )
                .await?;
        }
    }
    Ok(())
}
```

## ⚙️配置与扩展

限速：可在创建客户端时配置速率限制。

并发控制：支持自定义并发数与线程池。

生命周期钩子：在任务开始、结束、失败时注册回调函数。

下载进度监听：通过事件流获取实时进度。

热更新：配置文件变更后可自动应用，无需重启。

🧭 设计理念
本库专注于 底层能力与灵活性，而非强制策略。 例如：

文件冲突时如何处理（覆盖、跳过、重命名） → 由开发者决定

并发策略（FIFO、优先级队列等） → 由开发者实现

这样你可以根据业务需求自由扩展，而不是被框死在某种模式里。

## 📂 关于目录递归与响应式架构

为了实现真正的 **全局响应式机制**，本库不提供默认的全局递归目录遍历。  
这是为了避免在响应式系统中引入不可控的深层状态依赖。

如需递归获取目录内容，请使用 `get_folders` 方法：

- ✅ 支持传入 **文件夹路径与文件路径混用**。
- ✅ 支持指定递归深度（如 `Depth::One`、`Depth::Infinity`、`Depth::Zero`）。
- ✅ 返回结构化的资源列表，适用于后续下载、同步等操作。

这种设计使得开发者可以根据业务需求灵活控制递归行为，同时保持响应式系统的稳定性与可预测性。

示例代码：
```rust 
#[tokio::test]
async fn test_folders() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_2);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    let _ = client
        .get_folders(&key, &vec!["./文件夹1/".to_string(),"./文件夹2/core.exe".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
 ```

---

## 🌍 各语言 WebDAV 库对比

### 核心能力

| 语言 / 库                       | 基础操作 (GET/PUT/DELETE/MKCOL) | 许可证            |
|------------------------------|-----------------------------|----------------|
| Rust – 本库                    | ✅                           | MIT/Apache-2.0 |
| Java – Apache Jackrabbit     | ✅                           | Apache-2.0     |
| Python – PyWebDAV / pydav    | ✅                           | ZPL 2.0        |
| PHP – sabre/dav              | ✅                           | BSD            |
| C – neon                     | ✅                           | LGPL           |
| C++ – libdavix               | ✅                           | LGPL           |
| Go – golang.org/x/net/webdav | ✅                           | BSD            |
| JavaScript – webdav (npm)    | ✅                           | MIT            |

### 高级特性

| 语言 / 库                       | 断点续传 | 并发控制    | 响应式               | 下载进度 | 上传进度 | 生命周期钩子 | 热更新 |
|------------------------------|------|---------|-------------------|------|------|--------|-----|
| Rust – 本库                    | ✅    | ✅ 自定义   | ✅ (特性 `reactive`) | ✅    | ✅    | ✅      | ✅   |
| Java – Apache Jackrabbit     | ❌    | 部分（线程池） | ❌                 | ❌    | ❌    | ❌      | ❌   |
| Python – PyWebDAV / pydav    | ❌    | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |
| PHP – sabre/dav              | ❌    | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |
| C – neon                     | ❌    | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |
| C++ – libdavix               | 部分   | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |
| Go – golang.org/x/net/webdav | ❌    | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |
| JavaScript – webdav (npm)    | ❌    | ❌       | ❌                 | ❌    | ❌    | ❌      | ❌   |

> 说明：
> - 并发控制中的“部分（线程池）”表示可通过使用语言/框架的线程池或外部调度实现，库本身不内建策略。
> - 本库的响应式机制通过 `reactive` 特性开启，关闭时即为普通 WebDAV 客户端。
> - “断点续传”“进度”“钩子”“热更新”均为库的增强能力，便于构建下载器/同步器等复杂应用。
---

## 🧪 性能测试：响应式属性修改

为了验证响应式属性的性能，下面是针对**一个**响应式属性进行的高频修改测试：

| 项目       | 数值                        |
|----------|---------------------------|
| 总修改次数    | 1,000,000 次               |
| 总监听器数量   | 30 个                      |
| 总耗时      | 307.24 ms                 |
| 平均每次修改耗时 | ≈ 0.307 微秒                |
| 物理内存使用   | 5,857,280 bytes ≈ 5.58 MB |
| 虚拟内存使用   | 1,060,864 bytes ≈ 1.01 MB |

> 测试环境：单线程 Tokio runtime，默认配置，无限速，无 IO 操作。
>
> 主机配置：14600k（cpu）+ 64g 5600hz（内存） + 7650gre（显卡）
>
> 测试代码可参考: `src/tests/traits_impl_test/download.rs >>> test_reactive_data()`

### 🔥 性能亮点

- 响应式属性修改在百万级别操作下仍保持亚微秒级响应。
- 内存占用极低，适合嵌入式或资源受限场景。
- 支持高并发监听器，无明显性能瓶颈。

---

📌 TODO / 未来计划
[ ] 更丰富的错误类型与诊断信息

[ ] 上传任务支持

[ ] 更完善的文档与示例

---

参考资料：

- [WebDAV 软件比较 - 维基百科](https://zh.wikipedia.org/wiki/WebDAV%E8%BD%AF%E4%BB%B6%E6%AF%94%E8%BE%83)
- [Awesome WebDAV (GitHub)](https://github.com/fstanis/awesome-webdav)
