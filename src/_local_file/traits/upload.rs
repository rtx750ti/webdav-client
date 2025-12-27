use async_trait::async_trait;

#[async_trait]
pub trait Upload {
    async fn upload(&self, remote_path: &str) -> Result<(), String>;
}
