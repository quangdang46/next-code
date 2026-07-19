pub struct Syntect;
pub struct MarkdownStyle;
pub enum ColorLevel { Basic, TrueColor }
pub fn set_color_level_cap(_: ColorLevel) {}
pub fn render_markdown_ratatui_full(_md: &str, _style: &MarkdownStyle, _head: bool, _wid: Option<u16>) -> (String, usize) { (String::new(), 0) }
