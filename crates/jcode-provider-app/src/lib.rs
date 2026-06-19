pub mod catalog;
pub mod integration;
pub mod credential;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::*;
    use crate::integration::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_catalog_add_provider() {
        let mut cat = Catalog::new();
        cat.add_provider(ProviderEntry {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            enabled: true,
            is_connected: false,
        });
        assert_eq!(cat.providers().len(), 1);
    }

    #[test]
    fn test_integration_connection() {
        let mut int = Integration::new();
        int.set_connection("anthropic", ConnectionInfo::ApiKey { env_var: "ANTHROPIC_API_KEY".into() });
        assert!(int.has_any_credential("anthropic"));
        assert!(!int.has_any_credential("openai"));
    }
}
