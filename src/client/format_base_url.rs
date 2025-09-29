use url::Url;

pub fn format_base_url(url: &str) -> Result<Url, String> {
    if url.is_empty() {
        return Err("路径为空".to_string());
    }

    let mut base_url = Url::parse(url).map_err(|e| e.to_string())?;

    if !base_url.path().ends_with('/') {
        let new_path = format!("{}/", base_url.path());
        base_url.set_path(&new_path);
    }

    Ok(base_url)
}
