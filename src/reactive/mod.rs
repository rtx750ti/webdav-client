use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

#[derive(Clone, Debug)]
pub struct ReactiveProperty<T: Clone + Send + Sync> {
    inner: Arc<Inner<T>>,
}

impl<T> ReactiveProperty<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        let (sender, _) = watch::channel(Some(value));
        Self {
            inner: Arc::new(Inner {
                sender,
                is_dropped: AtomicBool::new(false),
            }),
        }
    }
    pub fn update(&self, new_value: T) -> Result<(), String> {
        if self.inner.is_dropped.load(Ordering::Relaxed) {
            Err("响应式属性已被销毁".to_string())
        } else {
            self.inner
                .sender
                .send(Some(new_value))
                .map_err(|e| format!("发送失败:{}", e.to_string()))?;

            Ok(())
        }
    }

    pub fn watch(&self) -> PropertyWatcher<T> {
        PropertyWatcher {
            receiver: self.inner.sender.subscribe(),
            inner: Arc::clone(&self.inner),
        }
    }

    pub fn get_current(&self) -> Option<T> {
        let temp_receiver = self.inner.sender.subscribe();
        temp_receiver.borrow().clone()
    }
}

#[derive(Debug)]
struct Inner<T> {
    sender: watch::Sender<Option<T>>,
    is_dropped: AtomicBool,
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        self.is_dropped.store(true, Ordering::Relaxed);
        let _ = self.sender.send(None);
    }
}

pub struct PropertyWatcher<T> {
    receiver: watch::Receiver<Option<T>>,
    inner: Arc<Inner<T>>,
}

impl<T> PropertyWatcher<T>
where
    T: Clone + Send + Sync,
{
    pub async fn changed(&mut self) -> Result<T, String> {
        self.receiver.changed().await.map_err(|e| e.to_string())?;
        match self.receiver.borrow().as_ref() {
            None => Err("监听器已被销毁".to_string()),
            Some(value) => Ok(value.clone()),
        }
    }
}
