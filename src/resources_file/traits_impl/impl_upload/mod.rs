mod handle_upload;
pub mod simple_upload;
pub mod chunked_upload;
pub mod local_file_upload;

use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::upload::{Upload, UploadConfig, FileOperations};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use crate::resources_file::traits_impl::impl_upload::handle_upload::{handle_upload, HandleUploadArgs};
use crate::public::enums::methods::WebDavMethod;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

/// 预处理上传路径，确保路径格式正确
fn preprocessing_upload_path(
    base_data: &Arc<ResourceFileData>,
    remote_path: &str,
) -> Result<String, String> {
    let base_url = &base_data.base_url;
    
    // 确保远程路径以 / 开头
    let normalized_path = if remote_path.starts_with('/') {
        remote_path.to_string()
    } else {
        format!("/{}", remote_path)
    };
    
    // 构建完整的上传 URL
    let upload_url = base_url.join(&normalized_path)
        .map_err(|e| format!("构建上传URL失败: {}", e))?;
    
    Ok(upload_url.to_string())
}

/// 创建文件夹的实现
async fn create_folder_impl(
    resources_file: &ResourcesFile,
    remote_path: &str,
) -> Result<(), String> {
    let upload_url = preprocessing_upload_path(
        &resources_file.get_data(),
        remote_path,
    )?;
    
    let http_client = resources_file.get_http_client();
    
    // 构建 MKCOL 请求
    let method = WebDavMethod::MKCOL
        .to_head_method()
        .map_err(|e| format!("构建MKCOL方法失败: {}", e))?;
    
    let response = http_client
        .request(method, &upload_url)
        .send()
        .await
        .map_err(|e| format!("发送MKCOL请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "创建文件夹失败: {} - {}",
            response.status(),
            upload_url
        ));
    }
    
    Ok(())
}

/// 删除文件或文件夹的实现
async fn delete_impl(resources_file: &ResourcesFile) -> Result<(), String> {
    let data = resources_file.get_data();
    let http_client = resources_file.get_http_client();
    
    // 构建 DELETE 请求
    let method = WebDavMethod::DELETE
        .to_head_method()
        .map_err(|e| format!("构建DELETE方法失败: {}", e))?;
    
    let response = http_client
        .request(method, &data.absolute_path)
        .send()
        .await
        .map_err(|e| format!("发送DELETE请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "删除失败: {} - {}",
            response.status(),
            data.absolute_path
        ));
    }
    
    Ok(())
}

/// 移动文件或文件夹的实现
async fn move_to_impl(
    resources_file: &ResourcesFile,
    destination_path: &str,
) -> Result<ResourcesFile, String> {
    let data = resources_file.get_data();
    let http_client = resources_file.get_http_client();
    
    let destination_url = preprocessing_upload_path(&data, destination_path)?;
    
    // 构建 MOVE 请求
    let method = WebDavMethod::MOVE
        .to_head_method()
        .map_err(|e| format!("构建MOVE方法失败: {}", e))?;
    
    let mut headers = HeaderMap::new();
    headers.insert("Destination", HeaderValue::from_str(&destination_url)
        .map_err(|e| format!("设置Destination头失败: {}", e))?);
    
    let response = http_client
        .request(method, &data.absolute_path)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("发送MOVE请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "移动失败: {} - {} -> {}",
            response.status(),
            data.absolute_path,
            destination_url
        ));
    }
    
    // 创建新的 ResourceFileData
    let mut new_data = (*data).clone();
    new_data.absolute_path = destination_url;
    new_data.name = destination_path.split('/').last().unwrap_or(destination_path).to_string();
    
    // 创建新的 ResourcesFile
    #[cfg(feature = "reactive")]
    let new_resources_file = ResourcesFile::new(
        new_data,
        http_client.clone(),
        resources_file.get_global_config(),
    );
    
    #[cfg(not(feature = "reactive"))]
    let new_resources_file = ResourcesFile::new(new_data, http_client.clone());
    
    Ok(new_resources_file)
}

/// 复制文件或文件夹的实现
async fn copy_to_impl(
    resources_file: &ResourcesFile,
    destination_path: &str,
) -> Result<ResourcesFile, String> {
    let data = resources_file.get_data();
    let http_client = resources_file.get_http_client();
    
    let destination_url = preprocessing_upload_path(&data, destination_path)?;
    
    // 构建 COPY 请求
    let method = WebDavMethod::COPY
        .to_head_method()
        .map_err(|e| format!("构建COPY方法失败: {}", e))?;
    
    let mut headers = HeaderMap::new();
    headers.insert("Destination", HeaderValue::from_str(&destination_url)
        .map_err(|e| format!("设置Destination头失败: {}", e))?);
    
    let response = http_client
        .request(method, &data.absolute_path)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("发送COPY请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "复制失败: {} - {} -> {}",
            response.status(),
            data.absolute_path,
            destination_url
        ));
    }
    
    // 创建新的 ResourceFileData
    let mut new_data = (*data).clone();
    new_data.absolute_path = destination_url;
    new_data.name = destination_path.split('/').last().unwrap_or(destination_path).to_string();
    
    // 创建新的 ResourcesFile
    #[cfg(feature = "reactive")]
    let new_resources_file = ResourcesFile::new(
        new_data,
        http_client.clone(),
        resources_file.get_global_config(),
    );
    
    #[cfg(not(feature = "reactive"))]
    let new_resources_file = ResourcesFile::new(new_data, http_client.clone());
    
    Ok(new_resources_file)
}

#[async_trait]
impl Upload for ResourcesFile {
    async fn upload_file<P: AsRef<Path> + Send>(
        &self,
        local_file_path: P,
        remote_path: &str,
        config: Option<UploadConfig>,
    ) -> Result<Arc<Self>, String> {
        let config = config.unwrap_or_default();
        
        let upload_url = preprocessing_upload_path(&self.get_data(), remote_path)?;
        
        let handle_upload_args = HandleUploadArgs {
            local_file_path: local_file_path.as_ref().to_path_buf(),
            upload_url,
            http_client: self.get_http_client().clone(),
            config,
            global_config: self.get_global_config(),
            #[cfg(feature = "reactive")]
            inner_state: self.get_reactive_state(),
            #[cfg(feature = "reactive")]
            inner_config: self.get_reactive_config(),
        };
        
        handle_upload(handle_upload_args)
            .await
            .map_err(|e| format!("[handle_upload] {}", e))?;
        
        Ok(Arc::new(self.clone()))
    }
    
    async fn upload_bytes(
        &self,
        data: Vec<u8>,
        remote_path: &str,
        config: Option<UploadConfig>,
    ) -> Result<Arc<Self>, String> {
        let _config = config.unwrap_or_default();
        
        let upload_url = preprocessing_upload_path(&self.get_data(), remote_path)?;
        let http_client = self.get_http_client();
        
        // 构建 PUT 请求
        let method = WebDavMethod::PUT
            .to_head_method()
            .map_err(|e| format!("构建PUT方法失败: {}", e))?;
        
        let response = http_client
            .request(method, &upload_url)
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(data)
            .send()
            .await
            .map_err(|e| format!("发送PUT请求失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!(
                "上传字节数据失败: {} - {}",
                response.status(),
                upload_url
            ));
        }
        
        Ok(Arc::new(self.clone()))
    }
    
    async fn create_folder(&self, remote_path: &str) -> Result<Arc<Self>, String> {
        create_folder_impl(self, remote_path).await?;
        Ok(Arc::new(self.clone()))
    }
}

#[async_trait]
impl FileOperations for ResourcesFile {
    async fn delete(&self) -> Result<(), String> {
        delete_impl(self).await
    }
    
    async fn move_to(&self, destination_path: &str) -> Result<Arc<Self>, String> {
        let new_file = move_to_impl(self, destination_path).await?;
        Ok(Arc::new(new_file))
    }
    
    async fn copy_to(&self, destination_path: &str) -> Result<Arc<Self>, String> {
        let new_file = copy_to_impl(self, destination_path).await?;
        Ok(Arc::new(new_file))
    }
    
    async fn rename(&self, new_name: &str) -> Result<Arc<Self>, String> {
        let data = self.get_data();
        let parent_path = data.absolute_path.rsplitn(2, '/').nth(1).unwrap_or("");
        let new_path = if parent_path.is_empty() {
            new_name.to_string()
        } else {
            format!("{}/{}", parent_path, new_name)
        };
        
        self.move_to(&new_path).await
    }
}
