use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait ApiKeyProvider: Send + Sync + 'static {
    fn current_api_key(&self) -> Option<String>;

    fn current_api_key_async(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        Box::pin(std::future::ready(self.current_api_key()))
    }
}

pub type SharedApiKeyProvider = Arc<dyn ApiKeyProvider>;
