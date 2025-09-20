use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

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
    /// 返回一个新的 `ReactiveProperty<T>` 实例。
    ///
    /// # 示例
    /// ```rust
    /// let prop = ReactiveProperty::new("Hello".to_string());
    /// assert_eq!(prop.get_current(), Some("Hello".to_string()));
    /// ```
    pub fn new(value: T) -> Self {
        let (sender, _) = watch::channel(Some(value));
        Self {
            inner: Arc::new(Inner {
                sender,
                is_dropped: AtomicBool::new(false),
            }),
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
    /// - `Ok(())`: 更新成功。
    /// - `Err(String)`: 如果属性已被销毁或发送失败。
    ///
    /// # 示例
    /// ```rust
    /// let prop = ReactiveProperty::new(10);
    /// prop.update(20).unwrap();
    /// assert_eq!(prop.get_current(), Some(20));
    /// ```
    pub fn update(&self, new_value: T) -> Result<&Self, String> {
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
    /// 返回一个 `PropertyWatcher<T>` 实例。
    ///
    /// # 示例
    /// ```rust
    /// use tokio::spawn;
    ///
    /// let prop = ReactiveProperty::new(1);
    /// let mut watcher = prop.watch();
    ///
    /// tokio::spawn(async move {
    ///     while let Ok(value) = watcher.changed().await {
    ///         println!("属性变化为: {}", value);
    ///     }
    /// });
    ///
    /// prop.update(2).unwrap();
    /// ```
    pub fn watch(&self) -> PropertyWatcher<T> {
        PropertyWatcher {
            receiver: self.inner.sender.subscribe(),
            inner: Arc::clone(&self.inner),
        }
    }

    /// 获取当前属性值的快照。
    ///
    /// # 返回值
    /// - `Some(T)`: 当前值。
    /// - `None`: 属性尚未初始化或已被销毁。
    ///
    /// # 示例
    /// ```rust
    /// let prop = ReactiveProperty::new("状态".to_string());
    /// let current = prop.get_current();
    ///
    /// assert_eq!(current, Some("状态".to_string()));
    /// ```
    pub fn get_current(&self) -> Option<T> {
        let temp_receiver = self.inner.sender.subscribe();
        temp_receiver.borrow().clone()
    }

    /// 使用闭包更新属性的部分字段。
    ///
    /// 适用于结构体字段的修改等场景。
    ///
    /// # 参数
    /// - `updater`: 一个闭包，接收当前值的可变引用并进行修改。
    ///
    /// # 返回值
    /// - `Ok(())`: 更新成功。
    /// - `Err(String)`: 如果属性未初始化或已被销毁。
    ///
    /// # 示例
    /// ```rust
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
    pub fn update_field<F, R>(&self, updater: F) -> Result<&Self, String>
    where
        F: FnOnce(&mut T) -> R,
    {
        if self.inner.is_dropped.load(Ordering::Relaxed) {
            // eprintln!("[ReactiveProperty] 已销毁，忽略 update_field");
            return Ok(self);
        }

        let mut current = match self.get_current() {
            Some(val) => val,
            None => {
                // eprintln!(
                //     "[ReactiveProperty] 当前值不存在，忽略 update_field"
                // );
                return Ok(self);
            }
        };

        updater(&mut current);

        match self.inner.sender.send(Some(current)) {
            Ok(_) => Ok(self),
            Err(_) => {
                // eprintln!(
                //     "[ReactiveProperty] 无接收者，update_field 更新被忽略"
                // );
                Ok(self)
            }
        }
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
    pub async fn changed(&mut self) -> Result<T, String> {
        self.receiver.changed().await.map_err(|e| e.to_string())?;
        match self.receiver.borrow().as_ref() {
            None => Err("监听器已被销毁".to_string()),
            Some(value) => Ok(value.clone()),
        }
    }

    /// 同步获取当前值的引用。
    pub fn borrow(&self) -> Option<T> {
        self.receiver.borrow().clone()
    }
}
