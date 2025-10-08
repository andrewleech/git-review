use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub struct SyntaxHighlighter {
    theme_name: String,
    // Note: Proper implementation should cache HighlightLines per file
    // for multi-line syntax support. Current implementation is simplified
    // and may not handle multi-line constructs correctly.
}

impl SyntaxHighlighter {
    pub fn new(theme_name: String) -> Self {
        Self { theme_name }
    }

    /// Get syntax highlighting for a code snippet
    ///
    /// Returns a vector of (text, Style) tuples for each segment
    pub fn highlight_line(&self, line: &str, file_extension: &str) -> Result<Vec<(String, Style)>> {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(file_extension)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let theme = THEME_SET
            .themes
            .get(&self.theme_name)
            .or_else(|| THEME_SET.themes.get("base16-ocean.dark"))
            .context("Could not load theme")?;

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();

        for line_text in LinesWithEndings::from(line) {
            let ranges = highlighter
                .highlight_line(line_text, &SYNTAX_SET)
                .context("Failed to highlight line")?;

            for (style, text) in ranges {
                result.push((text.to_string(), style));
            }
        }

        Ok(result)
    }

    /// Detect file extension from path
    pub fn detect_extension(file_path: &str) -> &str {
        file_path.rsplit('.').next().unwrap_or("txt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_extension() {
        assert_eq!(SyntaxHighlighter::detect_extension("main.rs"), "rs");
        assert_eq!(SyntaxHighlighter::detect_extension("test.py"), "py");
        assert_eq!(SyntaxHighlighter::detect_extension("README.md"), "md");
        assert_eq!(SyntaxHighlighter::detect_extension("noext"), "noext");
    }

    #[test]
    fn test_highlight_rust_code() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark".to_string());
        let code = "fn main() { println!(\"Hello\"); }";
        let result = highlighter.highlight_line(code, "rs");
        assert!(result.is_ok());
        let highlighted = result.unwrap();
        assert!(!highlighted.is_empty());
    }
}
