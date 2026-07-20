//! Minimal template renderer stub (no MiniJinja). Returns templates unchanged
//! except for trivial `${{ tools.by_kind.<key> }}` replacements.

use std::collections::HashMap;

use crate::types::tool::ToolKind;

#[derive(Debug, Clone)]
pub struct TemplateRenderError(pub String);

impl std::fmt::Display for TemplateRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TemplateRenderError {}

#[derive(Debug, Clone)]
pub struct TemplateRenderer {
    by_kind: HashMap<ToolKind, String>,
}

impl TemplateRenderer {
    pub fn new(
        tools: HashMap<ToolKind, String>,
        _params: HashMap<ToolKind, HashMap<String, String>>,
    ) -> Self {
        Self { by_kind: tools }
    }

    pub fn render(&self, template: &str) -> Result<String, TemplateRenderError> {
        let mut out = template.to_owned();
        for (kind, name) in &self.by_kind {
            let needle = format!("${{{{ tools.by_kind.{} }}}}", kind.as_key());
            out = out.replace(&needle, name);
        }
        Ok(out)
    }

    pub fn render_schema_descriptions(&self, _schema: &mut serde_json::Value) {}
}
