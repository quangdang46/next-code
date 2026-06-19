//! Concrete [`CredentialService`] implementations.

pub mod in_memory;
pub mod keyring;

pub use in_memory::InMemoryCredentialStore;
pub use keyring::KeyringCredentialStore;
