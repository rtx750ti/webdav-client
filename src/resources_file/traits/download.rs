use async_trait::async_trait;

#[async_trait]
pub trait Download {
    async fn download() -> Result<(), String>;
}
