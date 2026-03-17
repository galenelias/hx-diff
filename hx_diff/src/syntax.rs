use gpui::Hsla;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Clone, Debug)]
pub struct HighlightRun {
	pub byte_len: usize,
	pub color: Hsla,
}

pub struct SyntaxHighlighter {
	syntax_set: SyntaxSet,
	theme_set: ThemeSet,
}

impl SyntaxHighlighter {
	pub fn new() -> Self {
		Self {
			syntax_set: SyntaxSet::load_defaults_newlines(),
			theme_set: ThemeSet::load_defaults(),
		}
	}

	/// Highlights the given content using the syntax identified by the file extension.
	/// Returns a Vec where each element is the list of highlight runs for one line.
	pub fn highlight_content(&self, content: &str, file_path: &Path) -> Vec<Vec<HighlightRun>> {
		let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

		let syntax = self
			.syntax_set
			.find_syntax_by_extension(extension)
			.unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

		let theme = &self.theme_set.themes["base16-ocean.dark"];
		let mut highlighter = HighlightLines::new(syntax, theme);

		syntect::util::LinesWithEndings::from(content)
			.map(|line| {
				let ranges = highlighter
					.highlight_line(&line, &self.syntax_set)
					.unwrap_or_default();

				// Split off the last entry so we can cap the byte_len to not include the newline
				let ((last_style, last_text), elements) = ranges.split_last().unwrap();

				elements
					.iter()
					.map(|(style, text)| HighlightRun {
						byte_len: text.len(),
						color: syntect_color_to_hsla(style.foreground),
					})
					.chain(std::iter::once(HighlightRun {
						byte_len: last_text.len() - 1,
						color: syntect_color_to_hsla(last_style.foreground),
					}))
					.collect()
			})
			.collect()
	}
}

fn syntect_color_to_hsla(color: syntect::highlighting::Color) -> Hsla {
	Hsla::from(gpui::Rgba {
		r: color.r as f32 / 255.0,
		g: color.g as f32 / 255.0,
		b: color.b as f32 / 255.0,
		a: color.a as f32 / 255.0,
	})
}
