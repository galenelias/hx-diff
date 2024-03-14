use crate::*;
use gpui::*;
use theme::ActiveTheme;

fn byte_offset(substr: &str, outer_str: &str) -> usize {
	substr.as_ptr() as usize - outer_str.as_ptr() as usize
}

fn calculate_diff_highlights(
	diff: &str,
	cx: &WindowContext,
) -> Vec<(std::ops::Range<usize>, HighlightStyle)> {
	let removed_style = HighlightStyle {
		background_color: Some(cx.theme().status().deleted_background),
		color: Some(cx.theme().status().deleted),
		..Default::default()
	};
	let added_style = HighlightStyle {
		background_color: Some(cx.theme().status().created_background),
		color: Some(cx.theme().status().created),
		..Default::default()
	};
	let block_start = HighlightStyle {
		color: Some(opaque_grey(0.5, 1.0)),
		..Default::default()
	};

	let mut highlights = Vec::new();

	for line in diff.lines() {
		if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
			highlights.push((
				(byte_offset(line, diff)..byte_offset(line, diff) + line.len()),
				block_start,
			));
		} else if line.starts_with('+') {
			highlights.push((
				(byte_offset(line, diff)..byte_offset(line, diff) + line.len()),
				added_style,
			));
		} else if line.starts_with('-') {
			highlights.push((
				(byte_offset(line, diff)..byte_offset(line, diff) + line.len()),
				removed_style,
			));
		}
	}
	highlights
}

pub struct DiffPane {
	diff_text: SharedString,
}

impl DiffPane {
	pub fn new(_hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<DiffPane> {
		let file_list = cx.new_view(|_cx| DiffPane {
			diff_text: SharedString::from("Diff content goes here."),
		});

		file_list
	}

	pub fn open_diff(&mut self, filename: &str, cx: &mut ViewContext<Self>) {
		self.diff_text =
			SharedString::from(git_cli_wrap::get_diff(&filename).expect("Could not read file."));
	}
}

impl Render for DiffPane {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		let diff_text_style = TextStyle {
			color: cx.theme().colors().editor_foreground,
			font_family: "Menlo".into(),
			..Default::default()
		};

		div()
			.flex()
			.flex_col()
			.flex_1()
			.p_3()
			.id("DiffView")
			.overflow_x_scroll()
			.overflow_y_scroll()
			.bg(cx.theme().colors().editor_background)
			.text_color(cx.theme().colors().editor_foreground)
			.text_sm()
			.child(StyledText::new(self.diff_text.clone()).with_highlights(
				&diff_text_style,
				calculate_diff_highlights(&self.diff_text, cx),
			))
	}
}
