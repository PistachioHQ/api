/// Credentials for authenticating with the Pistachio Admin API.
///
/// Admin API operations require both an API key and a service account token.
#[derive(Debug, Clone)]
pub struct AdminCredentials {
    /// The API key for the project.
    api_key: String,
    /// The service account bearer token.
    service_account_token: String,
}

impl AdminCredentials {
    /// Creates new admin credentials.
    pub fn new(api_key: impl Into<String>, service_account_token: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            service_account_token: service_account_token.into(),
        }
    }

    /// Returns the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the service account token.
    pub fn service_account_token(&self) -> &str {
        &self.service_account_token
    }
}
