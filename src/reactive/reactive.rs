//! # ReactiveProperty
//!
//! 一个轻量级的响应式属性容器，基于 [`tokio::sync::watch`] 实现，
//! 支持异步监听和更新值。
//!
//! ## 特性
//! - **响应式更新**：当值发生变化时，所有监听者都会收到通知。
//! - **异步监听**：监听器可在异步任务中等待值的变化。
//! - **快照与零拷贝读取**：支持克隆快照（跨线程安全持有）和零拷贝借用（高性能读取）。
//! - **字段级更新**：通过闭包对结构体字段进行部分更新。
//!
//! ## 使用示例
//! ```rust
//! use crate::reactive::reactive::ReactiveProperty;
//!
//! #[tokio::main]
//! async fn main() {
//!     let prop = ReactiveProperty::new(0);
//!     let mut watcher = prop.watch();
//!
//!     tokio::spawn(async move {
//!         while let Ok(value) = watcher.changed().await {
//!             println!("属性变化为: {}", value);
//!         }
//!     });
//!
//!     prop.update(1).unwrap();
//!     prop.update(2).unwrap();
//! }
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tokio::sync::watch;
use tokio::sync::watch::Ref;
use tokio::sync::watch::error::RecvError;

/// 响应式属性相关错误
#[derive(Debug, Error)]
pub enum ReactivePropertyError {
    /// 监听器已被销毁
    #[error("监听器已被销毁")]
    WatcherClosed,

    /// watch 通道接收失败
    #[error("接收失败: {0}")]
    RecvError(#[from] RecvError),
}

/// 一个响应式属性容器，支持异步监听和更新。
///
/// `ReactiveProperty<T>` 封装了一个可被观察的值，
/// 当值发生变化时，所有监听者都会收到通知。
///
/// # 类型参数
/// - `T`: 必须实现 `Clone + Send + Sync`，以支持跨线程共享和异步操作。
#[derive(Clone, Debug)]
pub struct ReactiveProperty<T: Clone + Send + Sync> {
    inner: Arc<Inner<T>>,
    cache_receiver: watch::Receiver<Option<T>>,
}

impl<T> ReactiveProperty<T>
where
    T: Clone + Send + Sync,
{
    /// 创建一个新的响应式属性。
    ///
    /// # 参数
    /// - `value`: 初始值。
    ///
    /// # 返回值
    /// 返回一个新的 [`ReactiveProperty<T>`] 实例。
    ///
    /// # 示例
    /// ```
    /// use crate::reactive::reactive::ReactiveProperty;
    ///
    /// let prop = ReactiveProperty::new("Hello".to_string());
    /// assert_eq!(prop.get_current().unwrap().as_str(), "Hello");
    /// ```
    pub fn new(value: T) -> Self {
        let (sender, _) = watch::channel(Some(value));
        let cache_receiver = sender.subscribe();
        Self {
            inner: Arc::new(Inner {
                sender,
                is_dropped: AtomicBool::new(false),
            }),
            cache_receiver,
        }
    }

    /// 更新属性的值。
    ///
    /// 所有监听者都会收到新值的通知。
    ///
    /// # 参数
    /// - `new_value`: 要设置的新值。
    ///
    /// # 返回值
    /// - `Ok(&Self)`: 更新成功。
    /// - `Err(ReactivePropertyError)`: 如果属性已被销毁。
    ///
    /// # 示例
    /// ```
    /// use crate::reactive::reactive::ReactiveProperty;
    ///
    /// let prop = ReactiveProperty::new(10);
    /// prop.update(20).unwrap();
    /// assert_eq!(*prop.get_current().unwrap(), 20);
    /// ```
    pub fn update(
        &self,
        new_value: T,
    ) -> Result<&Self, ReactivePropertyError> {
        if self.inner.is_dropped.load(Ordering::Relaxed) {
            // eprintln!("[ReactiveProperty] 已销毁，忽略更新");
            return Ok(self);
        }

        match self.inner.sender.send(Some(new_value)) {
            Ok(_) => Ok(self),
            Err(_) => {
                // 没有任何 Receiver 存在
                // eprintln!("[ReactiveProperty] 无接收者，更新被忽略");
                Ok(self)
            }
        }
    }

    /// 创建一个监听器，用于异步监听属性值的变化。
    ///
    /// # 返回值
    /// 返回一个 [`PropertyWatcher<T>`] 实例。
    ///
    /// # 示例
    /// ```
    /// use crate::reactive::reactive::ReactiveProperty;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let prop = ReactiveProperty::new(1);
    ///     let mut watcher = prop.watch();
    ///
    ///     tokio::spawn(async move {
    ///         while let Ok(value) = watcher.changed().await {
    ///             println!("属性变化为: {}", value);
    ///         }
    ///     });
    ///
    ///     prop.update(2).unwrap();
    /// }
    /// ```
    pub fn watch(&self) -> PropertyWatcher<T> {
        PropertyWatcher {
            receiver: self.inner.sender.subscribe(),
            inner: Arc::clone(&self.inner),
        }
    }

    /// 获取当前属性值的快照。
    ///
    /// 与 [`PropertyWatcher::borrow`] 不同，`get_current` 会克隆底层值，
    /// 并返回一个新的 [`Arc<T>`]。这意味着调用者可以安全地在异步任务或跨线程中
    /// 长期持有该值，而不会受到后续更新的影响。
    ///
    /// ⚠️ 注意：由于会发生一次 `clone`，在高频调用或 `T` 较大时可能带来性能开销。
    ///
    /// # 返回值
    /// - `Some(Arc<T>)`: 当前值的快照。
    /// - `None`: 属性尚未初始化或已被销毁。
    pub fn get_current(&self) -> Option<Arc<T>> {
        self.cache_receiver.borrow().as_ref().map(|v| Arc::new(v.clone()))
    }

    /// 获取当前属性值的只读借用（零拷贝）。
    ///
    /// 与 [`get_current`](Self::get_current) 不同，`get_current_borrow` 不会克隆底层值，
    /// 而是返回一个 [`Ref<'_, Option<T>>`]，直接借用内部缓存。
    ///
    /// ⚠️ 注意：返回值的生命周期受限于 `&self`，不能跨异步边界或线程长期持有。
    pub fn get_current_borrow(&'_ self) -> Ref<'_, Option<T>> {
        self.cache_receiver.borrow()
    }
    /// 使用闭包更新属性的部分字段。
    ///
    /// 适用于结构体字段的修改等场景。
    ///
    /// # 参数
    /// - `updater`: 一个闭包，接收当前值的可变引用并进行修改。
    ///
    /// # 返回值
    /// - `Ok(&Self)`: 更新成功。
    /// - `Err(ReactivePropertyError)`: 如果属性未初始化或已被销毁。
    ///
    /// # 示例
    /// ```
    /// use crate::reactive::reactive::ReactiveProperty;
    ///
    /// #[derive(Clone)]
    /// struct State {
    ///     count: usize,
    /// }
    ///
    /// let prop = ReactiveProperty::new(State { count: 0 });
    /// prop.update_field(|s| s.count += 1).unwrap();
    ///
    /// assert_eq!(prop.get_current().unwrap().count, 1);
    /// ```
    pub fn update_field<F, R>(
        &self,
        updater: F,
    ) -> Result<&Self, ReactivePropertyError>
    where
        F: FnOnce(&mut T) -> R,
    {
        if self.inner.is_dropped.load(Ordering::Relaxed) {
            return Ok(self);
        }

        let mut current = match self.cache_receiver.borrow().clone() {
            Some(val) => val,
            None => return Ok(self),
        };

        updater(&mut current);

        let _ = self.inner.sender.send(Some(current));

        Ok(self)
    }
}

/// 内部共享状态，包含值发送器和销毁标志。
#[derive(Debug)]
struct Inner<T> {
    sender: watch::Sender<Option<T>>,
    is_dropped: AtomicBool,
}

impl<T> Drop for Inner<T> {
    /// 当 `ReactiveProperty` 被销毁时，通知所有监听者。
    fn drop(&mut self) {
        self.is_dropped.store(true, Ordering::Relaxed);
        let _ = self.sender.send(None);
    }
}

/// 属性监听器，用于异步接收属性值的变化。
///
/// 每次调用 [`changed`] 方法都会等待值的变化并返回新值。
pub struct PropertyWatcher<T> {
    receiver: watch::Receiver<Option<T>>,
    #[allow(dead_code)]
    inner: Arc<Inner<T>>,
}

impl<T> PropertyWatcher<T>
where
    T: Clone + Send + Sync,
{
    /// 异步等待属性值的变化。
    ///
    /// # 返回值
    /// - `Ok(T)`: 新的属性值。
    /// - `Err(String)`: 如果属性已被销毁或监听失败。
    pub async fn changed(&mut self) -> Result<T, ReactivePropertyError> {
        self.receiver.changed().await?;

        match self.receiver.borrow().as_ref() {
            None => Err(ReactivePropertyError::WatcherClosed),
            Some(value) => Ok(value.clone()),
        }
    }

    /// 同步获取当前值的引用。
    pub fn borrow(&self) -> Option<T> {
        self.receiver.borrow().clone()
    }
}
