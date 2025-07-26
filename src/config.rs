//! Configuration management for the rate limiter proxy

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub target_base_url: String,
    pub api_key: Option<String>,
    pub rate_limit: u32,
}

impl ProxyConfig {
    /// Load configuration from environment variables with validation
    pub fn from_env() -> Result<Self, ConfigError> {
        let target_base_url = std::env::var("TARGET_API_URL")
            .unwrap_or_else(|_| "https://api.example.com".to_string());
        
        let api_key = std::env::var("TARGET_API_KEY").ok();
        
        let rate_limit = std::env::var("RATE_LIMIT")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidRateLimit)?;

        // Validate URL format
        if !target_base_url.starts_with("http") {
            return Err(ConfigError::InvalidUrl);
        }

        Ok(Self {
            target_base_url,
            api_key,
            rate_limit,
        })
    }

    pub fn display_summary(&self) {
        println!("🚀 Starting API Rate Limiter Proxy");
        println!("📡 Target API: {}", self.target_base_url);
        println!("🚦 Rate Limit: {} requests per minute", self.rate_limit);
        println!("🔑 API Key: {}", if self.api_key.is_some() { "Configured" } else { "Not configured" });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid rate limit value")]
    InvalidRateLimit,
    #[error("Invalid URL format")]
    InvalidUrl,
} 