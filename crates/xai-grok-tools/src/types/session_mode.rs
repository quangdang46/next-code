#[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SessionMode {
    Default,
    Plan,
    Ask,
}

impl SessionMode {
    pub fn from_id(id: &str) -> Self {
        id.parse().unwrap_or(Self::Default)
    }

    pub fn as_id(&self) -> &'static str {
        self.into()
    }

    pub fn is_plan(&self) -> bool {
        matches!(self, Self::Plan)
    }
}
