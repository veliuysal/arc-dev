use thiserror::Error;
use url::Url;

pub const DEFAULT_RPC_URL: &str = "https://rpc.testnet.arc.network";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("RPC endpoint must be a valid HTTP or HTTPS URL")]
    InvalidRpcUrl,
}

pub fn resolve_rpc_url(cli_value: Option<String>) -> Result<Url, ConfigError> {
    let env_value = std::env::var("ARC_RPC_URL").ok();
    resolve_rpc_url_from(cli_value.as_deref(), env_value.as_deref())
}

fn resolve_rpc_url_from(
    cli_value: Option<&str>,
    env_value: Option<&str>,
) -> Result<Url, ConfigError> {
    let raw = cli_value
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env_value.filter(|value| !value.trim().is_empty()))
        .unwrap_or(DEFAULT_RPC_URL);

    let url = Url::parse(raw).map_err(|_| ConfigError::InvalidRpcUrl)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::InvalidRpcUrl);
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_testnet_when_no_override_is_set() {
        let url = resolve_rpc_url_from(None, None).unwrap();
        assert_eq!(url.as_str(), format!("{DEFAULT_RPC_URL}/"));
    }

    #[test]
    fn uses_cli_then_environment_then_testnet() {
        let cli =
            resolve_rpc_url_from(Some("http://localhost:8545"), Some("http://localhost:9545"))
                .unwrap();
        assert_eq!(cli.as_str(), "http://localhost:8545/");

        let env = resolve_rpc_url_from(Some(""), Some("http://localhost:9545")).unwrap();
        assert_eq!(env.as_str(), "http://localhost:9545/");
    }

    #[test]
    fn rejects_non_http_urls() {
        let error = resolve_rpc_url_from(Some("file:///tmp/rpc"), None).unwrap_err();
        assert!(matches!(error, ConfigError::InvalidRpcUrl));
    }
}
