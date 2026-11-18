use std::net::SocketAddr;
use url::Url;

#[derive(Clone)]
pub struct GatewaySettings {
    pub bind: SocketAddr,
    pub upstream: Url,
}

impl GatewaySettings {
    pub fn load() -> Result<Self, &'static str> {
        let bind = std::env::var("NEXORA_MCP_BIND")
            .unwrap_or_else(|_| "0.0.0.0:18081".to_owned())
            .parse()
            .map_err(|_| "invalid NEXORA_MCP_BIND")?;
        let raw =
            std::env::var("NEXORA_MCP_UPSTREAM").map_err(|_| "NEXORA_MCP_UPSTREAM is required")?;
        let upstream = Url::parse(&raw).map_err(|_| "invalid NEXORA_MCP_UPSTREAM")?;
        Self::validate_upstream(&upstream)?;
        Ok(Self { bind, upstream })
    }

    fn validate_upstream(upstream: &Url) -> Result<(), &'static str> {
        if !matches!(upstream.scheme(), "http" | "https")
            || upstream.host_str().is_none()
            || !matches!(upstream.path(), "/mcp" | "/mcp/")
            || upstream.username() != ""
            || upstream.password().is_some()
            || upstream.query().is_some()
            || upstream.fragment().is_some()
        {
            return Err(
                "NEXORA_MCP_UPSTREAM must be an HTTP(S) /mcp URL without credentials or query",
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::GatewaySettings;
    use url::Url;

    #[test]
    fn upstream_must_be_an_operator_configured_mcp_endpoint() {
        for value in [
            "http://nexora-api:18080/mcp",
            "https://api.example.invalid/mcp/",
        ] {
            let url = Url::parse(value).unwrap();
            assert!(GatewaySettings::validate_upstream(&url).is_ok());
        }
        for value in [
            "ftp://api.example.invalid/mcp",
            "http://user:password@api.example.invalid/mcp",
            "http://api.example.invalid/api",
            "http://api.example.invalid/mcp?target=other",
        ] {
            let url = Url::parse(value).unwrap();
            assert!(GatewaySettings::validate_upstream(&url).is_err());
        }
    }
}
