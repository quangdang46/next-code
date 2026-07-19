use std::path::{Path, PathBuf};

/// A syntax highlighter wrapping the Syntect library.
pub struct Syntect {
    /// The loaded syntax definitions.
    pub syntax_set: syntect::parsing::SyntaxSet,
    /// The loaded theme (if any).
    pub theme: Option<syntect::highlighting::Theme>,
    /// Path to the theme file that was loaded (if any).
    theme_file_path: Option<PathBuf>,
}

impl Syntect {
    /// Create a new `Syntect` instance.
    ///
    /// - `theme`: a theme name hint (e.g. `"grok-night"`).
    /// - `theme_file`: optional path to a `.tmTheme` file on disk.
    ///   When `None`, no theme is loaded (the caller can set one later).
    pub fn new(_theme: &str, theme_file: Option<&Path>) -> Self {
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let theme = theme_file
            .and_then(|p| syntect::highlighting::ThemeSet::get_theme(p).ok());
        let theme_file_path = theme_file.map(|p| p.to_path_buf());
        Self {
            syntax_set,
            theme,
            theme_file_path,
        }
    }

    /// Path to the directory containing the theme file (if one was loaded).
    pub fn theme_dir(&self) -> Option<PathBuf> {
        self.theme_file_path
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    }

    /// Path to the theme file that was loaded (if any).
    pub fn theme_file(&self) -> Option<PathBuf> {
        self.theme_file_path.clone()
    }
}

/// Markdown rendering style configuration.
///
/// Each field carries the `anstyle::Style` used to render that specific markdown
/// element. The `outer` / `inner` naming convention distinguishes the
/// delimiter/surrounding style from the content style.
pub struct MarkdownStyle {
    /// Body text style.
    pub text: anstyle::Style,
    /// Unchecked task list item (`[ ]`) style.
    pub task_unchecked: anstyle::Style,
    /// Checked task list item (`[x]`) style.
    pub task_checked: anstyle::Style,
    /// Table outer border / separator style.
    pub table_outer: anstyle::Style,
    /// Strong / bold delimiter style (e.g. `**` markers).
    pub strong_outer: anstyle::Style,
    /// Strong / bold content style.
    pub strong_inner: anstyle::Style,
    /// Strikethrough delimiter style (e.g. `~~` markers).
    pub strikethrough_outer: anstyle::Style,
    /// Strikethrough content style.
    pub strikethrough_inner: anstyle::Style,
    /// Horizontal rule (`---`) style.
    pub rule: anstyle::Style,
    /// Math / LaTeX content style.
    pub math: anstyle::Style,
    /// List item bullet / marker style.
    pub list_item: anstyle::Style,
    /// Link URL text style.
    pub link_url: anstyle::Style,
    /// Link title (tooltip) text style.
    pub link_title: anstyle::Style,
    /// Link visible text style.
    pub link_text: anstyle::Style,
    /// Link bracket delimiter style.
    pub link_outer: anstyle::Style,
    /// Inline code backtick delimiter style.
    pub inline_code_outer: anstyle::Style,
    /// Inline code content style.
    pub inline_code_inner: anstyle::Style,
    /// Heading delimiter style (one per level h1..=h6).
    pub heading_outer: [anstyle::Style; 6],
    /// Heading content style (one per level h1..=h6).
    pub heading_inner: [anstyle::Style; 6],
    /// Emphasis / italic delimiter style (e.g. `*` markers).
    pub emphasis_outer: anstyle::Style,
    /// Emphasis / italic content style.
    pub emphasis_inner: anstyle::Style,
    /// Fenced code block with no language tag — text style.
    pub code_untagged: anstyle::Style,
    /// Fenced code block outer delimiter style.
    pub code_outer: anstyle::Style,
    /// Fenced code block language label style.
    pub code_language: anstyle::Style,
    /// Fenced code block background fill style.
    pub code_background: anstyle::Style,
    /// Blockquote `>` marker style.
    pub blockquote_outer: anstyle::Style,
}

impl MarkdownStyle {
    /// Build a `MarkdownStyle` from a `toml::Value`.
    ///
    /// Each recognized key is parsed as an optional style string (e.g.
    /// `"bold red on #222"`). Unrecognized or `None` fields keep a default
    /// (plain) style rather than erroring.
    ///
    /// The `heading_outer` and `heading_inner` fields accept an array of six
    /// style strings, one per heading level, or a single string applied to all
    /// six levels.
    pub fn from_toml_value(_value: &toml::Value) -> Self {
        // Build from defaults — all plain styles.
        // A production implementation would walk _value.as_table() and
        // parse each style string via anstyle's Style::from_str() or a
        // custom parser.
        MarkdownStyle::default()
    }

    /// Merge another `MarkdownStyle` into this one.
    ///
    /// For each field, if `other`'s field is **not** the default (plain)
    /// `anstyle::Style`, it replaces `self`'s value. This allows partial
    /// overrides from theme config files.
    pub fn merge(&mut self, other: &MarkdownStyle) {
        fn is_plain(s: &anstyle::Style) -> bool {
            s.get_fg_color().is_none()
                && s.get_bg_color().is_none()
                && s.get_effects().is_plain()
        }
        macro_rules! merge_field {
            ($field:ident) => {
                if !is_plain(&other.$field) {
                    self.$field = other.$field;
                }
            };
        }
        merge_field!(text);
        merge_field!(task_unchecked);
        merge_field!(task_checked);
        merge_field!(table_outer);
        merge_field!(strong_outer);
        merge_field!(strong_inner);
        merge_field!(strikethrough_outer);
        merge_field!(strikethrough_inner);
        merge_field!(rule);
        merge_field!(math);
        merge_field!(list_item);
        merge_field!(link_url);
        merge_field!(link_title);
        merge_field!(link_text);
        merge_field!(link_outer);
        merge_field!(inline_code_outer);
        merge_field!(inline_code_inner);
        merge_field!(emphasis_outer);
        merge_field!(emphasis_inner);
        merge_field!(code_untagged);
        merge_field!(code_outer);
        merge_field!(code_language);
        merge_field!(code_background);
        merge_field!(blockquote_outer);

        // heading arrays: merge element-by-element
        for i in 0..6 {
            if !is_plain(&other.heading_outer[i]) {
                self.heading_outer[i] = other.heading_outer[i];
            }
            if !is_plain(&other.heading_inner[i]) {
                self.heading_inner[i] = other.heading_inner[i];
            }
        }
    }
}

impl Default for MarkdownStyle {
    fn default() -> Self {
        Self {
            text: anstyle::Style::new(),
            task_unchecked: anstyle::Style::new(),
            task_checked: anstyle::Style::new(),
            table_outer: anstyle::Style::new(),
            strong_outer: anstyle::Style::new(),
            strong_inner: anstyle::Style::new(),
            strikethrough_outer: anstyle::Style::new(),
            strikethrough_inner: anstyle::Style::new(),
            rule: anstyle::Style::new(),
            math: anstyle::Style::new(),
            list_item: anstyle::Style::new(),
            link_url: anstyle::Style::new(),
            link_title: anstyle::Style::new(),
            link_text: anstyle::Style::new(),
            link_outer: anstyle::Style::new(),
            inline_code_outer: anstyle::Style::new(),
            inline_code_inner: anstyle::Style::new(),
            heading_outer: [anstyle::Style::new(); 6],
            heading_inner: [anstyle::Style::new(); 6],
            emphasis_outer: anstyle::Style::new(),
            emphasis_inner: anstyle::Style::new(),
            code_untagged: anstyle::Style::new(),
            code_outer: anstyle::Style::new(),
            code_language: anstyle::Style::new(),
            code_background: anstyle::Style::new(),
            blockquote_outer: anstyle::Style::new(),
        }
    }
}

/// Color level for terminal output.
pub enum ColorLevel {
    Basic,
    TrueColor,
}

/// Set the color level cap.
pub fn set_color_level_cap(_: ColorLevel) {}

/// Render markdown to ratatui-compatible output (minimal stub).
pub fn render_markdown_ratatui_full(
    _md: &str,
    _style: &MarkdownStyle,
    _head: bool,
    _wid: Option<u16>,
) -> (String, usize) {
    (String::new(), 0)
}
