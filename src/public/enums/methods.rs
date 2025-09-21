use reqwest::Method;

pub enum WebDavMethod {
    PROPFIND,
    PUT,
    DELETE,
    MKCOL,
    MOVE,
    COPY,
    PROPPATCH,
    LOCK,
    UNLOCK,
}

impl WebDavMethod {
    pub fn to_string(&self) -> String {
        match self {
            WebDavMethod::PROPFIND => "PROPFIND".to_string(),
            WebDavMethod::PUT => "PUT".to_string(),
            WebDavMethod::DELETE => "DELETE".to_string(),
            WebDavMethod::MKCOL => "MKCOL".to_string(),
            WebDavMethod::MOVE => "MOVE".to_string(),
            WebDavMethod::COPY => "COPY".to_string(),
            WebDavMethod::PROPPATCH => "PROPPATCH".to_string(),
            WebDavMethod::LOCK => "LOCK".to_string(),
            WebDavMethod::UNLOCK => "UNLOCK".to_string(),
        }
    }

    pub fn to_head_method(&self) -> Result<Method, String> {
        let method =
            reqwest::Method::from_bytes(self.to_string().as_bytes())
                .map_err(|e| e.to_string())?;

        match self {
            WebDavMethod::PROPFIND => Ok(method),
            WebDavMethod::PUT => Ok(method),
            WebDavMethod::DELETE => Ok(method),
            WebDavMethod::MKCOL => Ok(method),
            WebDavMethod::MOVE => Ok(method),
            WebDavMethod::COPY => Ok(method),
            WebDavMethod::PROPPATCH => Ok(method),
            WebDavMethod::LOCK => Ok(method),
            WebDavMethod::UNLOCK => Ok(method),
        }
    }
}
