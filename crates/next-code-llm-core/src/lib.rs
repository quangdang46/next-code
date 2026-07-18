pub mod auth;
pub mod endpoint;
pub mod framing;
pub mod model_ref;
pub mod protocol;
pub mod route;
pub mod schema;
pub mod transport;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
    #[test]
    fn test_auth_works() {
        use crate::auth::Auth;
        let mut req = crate::auth::Request {
            method: "GET".to_string(),
            url: "http://test".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        };
        let auth = crate::auth::bearer("token123".to_string());
        let result = futures::executor::block_on(auth.apply(&mut req));
        assert!(result.is_ok());
        assert_eq!(req.headers.get("Authorization").unwrap(), "Bearer token123");
    }
}
