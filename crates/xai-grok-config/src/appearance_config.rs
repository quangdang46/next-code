//! Appearance configuration types (stub).
//! Re-exports `toml::Value` as `RawAppearanceConfig` so callers can
//! navigate it with `.get()`, `.as_table()`, `.as_str()`, etc.

/// Raw appearance configuration, backed by a TOML value.
pub type RawAppearanceConfig = toml::Value;
