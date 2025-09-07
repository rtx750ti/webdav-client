use async_trait::async_trait;

#[async_trait]
pub trait Control {
    async fn stop(&self) -> Result<(), String>;
    async fn restart(&self) -> Result<(), String>;
}
