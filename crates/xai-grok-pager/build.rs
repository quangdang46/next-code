fn main() {
    // Upstream sets this from Bazel/release metadata. For next-code Face
    // builds, fall back to the package version so `env!("VERSION_WITH_COMMIT")`
    // resolves during `cargo check`.
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    println!("cargo:rustc-env=VERSION_WITH_COMMIT={version}");
}
