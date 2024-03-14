use crate::*;
use gpui::*;

pub struct HxDiff {
	weak_self: WeakView<Self>,
	file_list: View<FileList>,
	text: SharedString,
}

impl HxDiff {
	pub fn new(cx: &mut ViewContext<Self>) -> HxDiff {
		let weak_handle = cx.view().downgrade();
		let file_list = FileList::new(weak_handle.clone(), cx);

		cx.subscribe(&file_list, {
			move |subscriber, _, event, cx| match event {
				&FileListEvent::OpenedEntry { ref filename } => {
					subscriber.open_file(filename, cx);
				}
			}
		})
		.detach();

		HxDiff {
			weak_self: weak_handle,
			file_list,
			text: SharedString::from("Diff content goes here."),
		}
	}

	pub fn _weak_handle(&self) -> WeakView<Self> {
		self.weak_self.clone()
	}

	fn open_file(&mut self, filename: &str, cx: &mut ViewContext<Self>) {
		self.text =
			SharedString::from(git_cli_wrap::get_diff(&filename).expect("Could not read file."));
		cx.refresh();
	}
}

impl Render for HxDiff {
	fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.size_full()
			.flex()
			.flex_col()
			.child(
				div() // main status bar
					.flex_grow()
					.flex()
					.flex_row()
					.child(self.file_list.clone())
					.child(
						div().flex_grow().bg(rgb(0xa8dadc)).child(
							div()
								.p_2()
								.id("DiffView")
								.overflow_y_scroll()
								.child(self.text.clone()),
						),
					),
			)
			.child(
				div() // Status bar
					.min_h(px(30.0))
					.bg(rgb(0x1d3557))
					.child("Status Bar"),
			)
	}
}

// impl EventEmitter<UIEvent> for FileList {}
