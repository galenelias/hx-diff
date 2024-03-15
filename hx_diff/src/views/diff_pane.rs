use crate::*;
use gpui::{prelude::FluentBuilder, *};
use theme::ActiveTheme;

enum DiffType {
	Header,
	Normal,
	Added,
	Removed,
}

struct DiffLine {
	text: SharedString,
	diff_type: DiffType,
}

fn process_diff(diff: &str) -> Vec<DiffLine> {
	let mut lines = Vec::new();
	for line in diff.lines() {
		if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
			lines.push(DiffLine {
				text: line.to_string().into(),
				diff_type: DiffType::Header,
			});
		} else if line.starts_with('+') {
			lines.push(DiffLine {
				text: line[1..].to_string().into(),
				diff_type: DiffType::Added,
			});
		} else if line.starts_with('-') {
			lines.push(DiffLine {
				text: line[1..].to_string().into(),
				diff_type: DiffType::Removed,
			});
		} else {
			lines.push(DiffLine {
				text: line[1..].to_string().into(),
				diff_type: DiffType::Normal,
			});
		}
	}
	lines
}

pub struct DiffPane {
	diff_text: SharedString,
	diff_lines: Vec<DiffLine>,
}

impl DiffPane {
	pub fn new(_hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<DiffPane> {
		let file_list = cx.new_view(|_cx| DiffPane {
			diff_text: SharedString::from("Diff content goes here."),
			diff_lines: Vec::new(),
		});

		file_list
	}

	pub fn open_diff(&mut self, filename: &str, cx: &mut ViewContext<Self>) {
		self.diff_text =
			SharedString::from(git_cli_wrap::get_diff(&filename).expect("Could not read file."));

		self.diff_lines = process_diff(&self.diff_text);
	}

	fn render_diff_line(&self, item: &DiffLine, cx: &mut ViewContext<Self>) -> Div {
		let color = match item.diff_type {
			DiffType::Header => opaque_grey(0.5, 1.0),
			DiffType::Normal => cx.theme().colors().editor_foreground,
			DiffType::Added => cx.theme().status().created,
			DiffType::Removed => cx.theme().status().deleted,
		};

		let background_color = match item.diff_type {
			DiffType::Header => cx.theme().colors().editor_background,
			DiffType::Normal => cx.theme().colors().editor_background,
			DiffType::Added => cx.theme().status().created_background,
			DiffType::Removed => cx.theme().status().deleted_background,
		};

		let border = match item.diff_type {
			DiffType::Header => Some(px(3.)),
			_ => None,
		};

		div()
			.flex()
			.flex_row()
			.flex_grow()
			.w_full()
			.bg(background_color)
			.pl_3()
			// .border_t_width(px(3.))
			// .border_color(cx.theme().colors().editor_background)
			// .when_some(border, |el, border| {
			// 	el.border_t_width(border)
			// 		.border_color(cx.theme().colors().border)
			// })
			.hover(|s| s.bg(cx.theme().colors().element_hover))
			.child(
				div()
					.flex()
					.flex_grow()
					.flex_nowrap()
					.overflow_x_hidden()
					.text_color(color)
					.child(item.text.clone()),
			)
	}
}

impl Render for DiffPane {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.flex_1()
			// .pl_3()
			// .p_3()
			.id("DiffView")
			.bg(cx.theme().colors().editor_background)
			.text_sm()
			.font("Menlo")
			.child(uniform_list(
				cx.view().clone(),
				"entries",
				self.diff_lines.len(),
				{
					|this, range, cx| {
						range
							.map(|i| this.render_diff_line(&this.diff_lines[i], cx))
							.collect()
					}
				},
			))
	}
}
