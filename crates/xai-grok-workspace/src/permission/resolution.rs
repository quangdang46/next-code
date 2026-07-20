//! Permission resolution / managed-settings stubs (Face compile stubs).

/// Toast / pin copy when YOLO is forced off by requirements (test constant).
pub const YOLO_PIN_REASON_REQUIREMENTS: &str = "disabled by requirements";

/// Org-managed permission settings snapshot.
#[derive(Debug, Clone, Default)]
pub struct ManagedSettings {
    pub marketplace_allowlist: MarketplaceAllowlist,
}

/// Marketplace source allowlist from managed/requirements policy.
#[derive(Debug, Clone, Default)]
pub struct MarketplaceAllowlist {
    /// When true, only URLs matching the allowlist may be added.
    restricted: bool,
}

impl MarketplaceAllowlist {
    /// Unrestricted stub (local Face builds do not enforce org allowlists).
    pub fn unrestricted() -> Self {
        Self { restricted: false }
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }

    pub fn is_url_allowed(&self, _identity: &str) -> bool {
        !self.restricted
    }

    pub fn block_reason(&self) -> String {
        "marketplace source not on the managed allowlist".to_owned()
    }
}

/// Load managed permission settings. Stub returns unrestricted defaults.
pub fn managed_settings() -> ManagedSettings {
    ManagedSettings {
        marketplace_allowlist: MarketplaceAllowlist::unrestricted(),
    }
}