use crate::client::structs::client_key::ClientKey;
use crate::client::traits::url_format::{FormatUrlPathError, UrlFormat, UrlFormatError};
use crate::client::WebDavClient;


impl UrlFormat for WebDavClient {
    fn format_url_path(
        &self,
        key: &ClientKey,
        path: &str,
    ) -> Result<String, UrlFormatError> {
        let base_url = key.get_base_url();
        let joined_url = base_url.join(path).map_err(|e| {
            UrlFormatError::FormatUrlPathError(
                FormatUrlPathError::FormatError(e.to_string()),
            )
        })?;

        let err = Err(UrlFormatError::FormatUrlPathError(
            FormatUrlPathError::ParentDirNotAllowed,
        ));

        if !joined_url.as_str().starts_with(base_url.as_str()) {
            return err;
        }

        if joined_url.scheme() != base_url.scheme()
            || joined_url.host_str() != base_url.host_str()
            || !joined_url.path().starts_with(base_url.path())
        {
            return err;
        }

        Ok(joined_url.to_string())
    }
}
