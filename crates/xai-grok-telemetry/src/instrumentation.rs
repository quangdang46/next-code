pub fn layer() -> tracing_subscriber::layer::Identity {
    tracing_subscriber::layer::Identity::new()
}

pub fn init() {}

pub fn install_panic_hook() {}

pub fn finalize() -> Result<(), String> {
    Ok(())
}
