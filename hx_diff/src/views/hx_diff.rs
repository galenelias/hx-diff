use gpui::*;

use crate::*;
// use git_cli_wrap;

pub struct HxDiff {
	weak_self: WeakView<Self>,
	file_list: View<FileList>,
	text: SharedString,
}

impl HxDiff {
	pub fn new(cx: &mut ViewContext<Self>) -> HxDiff {
		let weak_handle = cx.view().downgrade();
		let file_list = FileList::new(weak_handle.clone(), cx);

		HxDiff {
			weak_self: weak_handle,
			file_list,
			text: SharedString::from("Diff content goes here."),
		}
	}

	pub fn weak_handle(&self) -> WeakView<Self> {
		self.weak_self.clone()
	}
}

impl Render for HxDiff {
	fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.size_full()
			// .size(Length::Definite(Pixels(300.0).into()))
			.flex()
			.flex_col()
			.child(
				div() // main status bar
					.flex_grow()
					.flex()
					.flex_row()
					.child(self.file_list.clone())
					.child(div().flex_grow().bg(rgb(0xa8dadc)).child(self.text.clone())),
			)
			.child(
				div() // Status bar
					.min_h(px(30.0))
					.bg(rgb(0x1d3557))
					.child("Status Bar"),
			)
	}
}
