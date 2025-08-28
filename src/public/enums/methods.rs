use reqwest::Method;

pub enum WebDavMethod {
    PROPFIND,
}

impl WebDavMethod {
    pub fn to_string(&self) -> String {
        match self {
            WebDavMethod::PROPFIND => "PROPFIND".to_string(),
        }
    }

    pub fn to_head_method(&self) -> Result<Method, String> {
        let method =
            reqwest::Method::from_bytes(self.to_string().as_bytes())
                .map_err(|e| e.to_string())?;

        match self {
            WebDavMethod::PROPFIND => Ok(method),
        }
    }
}
