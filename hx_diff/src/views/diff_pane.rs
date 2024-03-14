use crate::*;
use gpui::*;
use theme::ActiveTheme;

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
			.child(self.diff_text.clone())
	}
}
