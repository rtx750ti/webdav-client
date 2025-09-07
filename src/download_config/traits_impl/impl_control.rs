use crate::download_config::DownloadConfig;
use crate::download_config::traits::control::Control;
use async_trait::async_trait;

#[async_trait]
impl Control for DownloadConfig {
    async fn stop(&self) -> Result<(), String> {
        self.abort_handle.abort();
        println!("下载已停止");
        Ok(())
    }

    async fn restart(&self) -> Result<(), String> {
        todo!()
    }
}
