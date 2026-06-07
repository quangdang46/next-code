//! OS keychain abstraction layer.
//!
//! Provides a [`KeyringStore`] trait for storing/retrieving secrets via the
//! platform-native credential manager (macOS Keychain, Linux Secret Service /
//! GNOME Keyring, Windows Credential Manager).
//!
//! - [`DefaultKeyringStore`] — real OS keychain via the `keyring` crate
//! - [`MockKeyringStore`] — in-memory HashMap for tests, with optional error injection

use std::sync::Mutex;

/// OS keychain abstraction for storing/retrieving arbitrary secrets.
///
/// Each secret is identified by a (service, account) pair — the same
/// convention used by the `keyring` crate and platform-native tools like
/// `security(1)` on macOS.
pub trait KeyringStore: Send + Sync {
    /// Load a secret from the keychain.
    ///
    /// Returns `Ok(None)` when no entry exists for the given
    /// (service, account) pair.
    fn load(&self, service: &str, account: &str) -> anyhow::Result<Option<String>>;

    /// Save (create or overwrite) a secret in the keychain.
    fn save(&self, service: &str, account: &str, value: &str) -> anyhow::Result<()>;

    /// Delete a secret from the keychain.
    ///
    /// Returns `Ok(())` even when the entry does not exist (idempotent
    /// delete).
    fn delete(&self, service: &str, account: &str) -> anyhow::Result<()>;
}

// ─── DefaultKeyringStore ─────────────────────────────────────────────────────

/// Real OS keychain implementation via the `keyring` crate.
///
/// Uses the platform-native credential backend:
/// - macOS:  Apple Keychain (via Security.framework)
/// - Linux:  Secret Service / GNOME Keyring (via D-Bus)
/// - Windows: Credential Manager (via Win32 API)
pub struct DefaultKeyringStore;

impl DefaultKeyringStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultKeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyringStore for DefaultKeyringStore {
    fn load(&self, service: &str, account: &str) -> anyhow::Result<Option<String>> {
        let entry = keyring::Entry::new(service, account)?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&self, service: &str, account: &str, value: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(service, account)?;
        entry.set_password(value)?;
        Ok(())
    }

    fn delete(&self, service: &str, account: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new(service, account)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // idempotent delete
            Err(e) => Err(e.into()),
        }
    }
}

// ─── MockKeyringStore ────────────────────────────────────────────────────────

/// In-memory keychain mock for testing.
///
/// Supports optional error injection via [`set_inject_error`] — when enabled
/// every operation fails with `"injected error"`.
pub struct MockKeyringStore {
    store: Mutex<std::collections::HashMap<(String, String), String>>,
    inject_error: std::sync::atomic::AtomicBool,
}

impl MockKeyringStore {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(std::collections::HashMap::new()),
            inject_error: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// When `inject` is true all subsequent load/save/delete calls will fail.
    pub fn set_inject_error(&self, inject: bool) {
        self.inject_error
            .store(inject, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for MockKeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyringStore for MockKeyringStore {
    fn load(&self, service: &str, account: &str) -> anyhow::Result<Option<String>> {
        if self.inject_error.load(std::sync::atomic::Ordering::SeqCst) {
            anyhow::bail!("injected error");
        }
        let store = self.store.lock().unwrap();
        Ok(store
            .get(&(service.to_string(), account.to_string()))
            .cloned())
    }

    fn save(&self, service: &str, account: &str, value: &str) -> anyhow::Result<()> {
        if self.inject_error.load(std::sync::atomic::Ordering::SeqCst) {
            anyhow::bail!("injected error");
        }
        let mut store = self.store.lock().unwrap();
        store.insert(
            (service.to_string(), account.to_string()),
            value.to_string(),
        );
        Ok(())
    }

    fn delete(&self, service: &str, account: &str) -> anyhow::Result<()> {
        if self.inject_error.load(std::sync::atomic::Ordering::SeqCst) {
            anyhow::bail!("injected error");
        }
        let mut store = self.store.lock().unwrap();
        store.remove(&(service.to_string(), account.to_string()));
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_keyring_store_load_save_roundtrip() {
        let store = MockKeyringStore::new();
        store.save("test-svc", "test-acct", "secret-value").unwrap();
        let value = store.load("test-svc", "test-acct").unwrap();
        assert_eq!(value, Some("secret-value".to_string()));
    }

    #[test]
    fn mock_keyring_store_load_nonexistent() {
        let store = MockKeyringStore::new();
        let value = store.load("unknown", "nothing").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn mock_keyring_store_delete_nonexistent() {
        let store = MockKeyringStore::new();
        // Deleting a non-existent entry should not error (idempotent)
        store.delete("unknown", "nothing").unwrap();
    }

    #[test]
    fn mock_keyring_store_delete_removes_entry() {
        let store = MockKeyringStore::new();
        store.save("svc", "acct", "val").unwrap();
        store.delete("svc", "acct").unwrap();
        let value = store.load("svc", "acct").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn mock_keyring_store_error_injection() {
        let store = MockKeyringStore::new();
        store.set_inject_error(true);

        assert!(store.load("x", "y").is_err());
        assert!(store.save("x", "y", "z").is_err());
        assert!(store.delete("x", "y").is_err());

        store.set_inject_error(false);
        assert!(store.save("x", "y", "z").is_ok());
    }

    /// Verify that different (service, account) pairs are isolated.
    #[test]
    fn mock_keyring_store_isolation() {
        let store = MockKeyringStore::new();
        store.save("svc1", "acct1", "val1").unwrap();
        store.save("svc2", "acct2", "val2").unwrap();

        assert_eq!(
            store.load("svc1", "acct1").unwrap(),
            Some("val1".to_string())
        );
        assert_eq!(
            store.load("svc2", "acct2").unwrap(),
            Some("val2".to_string())
        );
        assert_eq!(store.load("svc1", "acct2").unwrap(), None);
    }
}
