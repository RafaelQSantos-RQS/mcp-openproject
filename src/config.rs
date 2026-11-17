use std::env;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable '{0}' (or specify via CLI argument)")]
    MissingVar(String),

    #[error("Invalid URL '{0}': {1}")]
    InvalidUrl(String, url::ParseError),

    #[error("URL must use http or https scheme, got: '{0}'")]
    InvalidScheme(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Normalized base URL (e.g. "https://ndsi-projects.senaicimatec.com.br")
    pub base_url: String,
    /// OpenProject API Key / Personal Access Token
    pub api_key: String,
}

impl Config {
    /// Loads configuration from environment variables (or .env file)
    /// with optional overrides from CLI arguments.
    pub fn load(cli_url: Option<String>, cli_key: Option<String>) -> Result<Self, ConfigError> {
        // Load .env if present, ignore if not found
        dotenvy::dotenv().ok();

        let raw_url = cli_url
            .or_else(|| env::var("OPENPROJECT_URL").ok())
            .ok_or_else(|| ConfigError::MissingVar("OPENPROJECT_URL".into()))?;

        let api_key = cli_key
            .or_else(|| env::var("OPENPROJECT_API_KEY").ok())
            .ok_or_else(|| ConfigError::MissingVar("OPENPROJECT_API_KEY".into()))?;

        Self::new(raw_url, api_key)
    }

    /// Creates and validates a Config instance from raw URL and API Key
    pub fn new(raw_url: String, api_key: String) -> Result<Self, ConfigError> {
        let trimmed_url = raw_url.trim().trim_end_matches('/');

        // Strip trailing /api/v3 or /api if the user provided it in OPENPROJECT_URL
        let normalized = if let Some(stripped) = trimmed_url.strip_suffix("/api/v3") {
            stripped.trim_end_matches('/')
        } else if let Some(stripped) = trimmed_url.strip_suffix("/api") {
            stripped.trim_end_matches('/')
        } else {
            trimmed_url
        };

        let parsed =
            Url::parse(normalized).map_err(|e| ConfigError::InvalidUrl(raw_url.clone(), e))?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(ConfigError::InvalidScheme(parsed.scheme().into()));
        }

        Ok(Self {
            base_url: normalized.to_string(),
            api_key: api_key.trim().to_string(),
        })
    }

    /// Formats the full API v3 endpoint for a relative subpath
    pub fn api_v3_url(&self, path: &str) -> String {
        let clean_path = path.trim_start_matches('/');
        if clean_path.starts_with("api/v3") {
            format!("{}/{}", self.base_url, clean_path)
        } else {
            format!("{}/api/v3/{}", self.base_url, clean_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_normalization() {
        let c1 = Config::new("https://example.com/".into(), "token123".into()).unwrap();
        assert_eq!(c1.base_url, "https://example.com");
        assert_eq!(
            c1.api_v3_url("work_packages"),
            "https://example.com/api/v3/work_packages"
        );
        assert_eq!(
            c1.api_v3_url("/api/v3/projects"),
            "https://example.com/api/v3/projects"
        );

        let c2 = Config::new("https://example.com/api/v3/".into(), "token123".into()).unwrap();
        assert_eq!(c2.base_url, "https://example.com");
        assert_eq!(c2.api_v3_url("types"), "https://example.com/api/v3/types");
    }

    #[test]
    fn test_invalid_scheme() {
        let err = Config::new("ftp://example.com".into(), "token".into()).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidScheme(_)));
    }
}
