use crate::*;
use gpui::prelude::*;
use gpui::*;
use theme::ActiveTheme;

#[derive(Clone)]
pub enum PanelPosition {
	Left,
	// Right,
}

#[derive(Clone, Render)]
pub struct DraggedPanel(pub PanelPosition);

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
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.size_full()
			.flex()
			.flex_col()
			.text_color(cx.theme().colors().text)
			.child(
				div() // main status bar
					.flex_grow()
					.flex()
					.flex_row()
					.h_full()
					.min_h_0() // Prevent the height from auto-fitting the children
					.on_drag_move(
						cx.listener(move |this, e: &DragMoveEvent<DraggedPanel>, cx| {
							// println!("on_drag_move! {:?}", e.event.position);
							match e.drag(cx).0 {
								PanelPosition::Left => {
									let size = /*this.bounds.left() +*/ e.event.position.x;
									this.file_list.update(cx, |file_list, cx| {
										file_list.resize_panel(Some(size), cx);
									});
								}
							}
						}),
					)
					.child(self.file_list.clone())
					.child(
						div()
							.flex()
							.flex_col()
							// .size_full()
							// .flex_grow()
							// .h_full()
							.flex_1()
							.p_3()
							.id("DiffView")
							.overflow_x_scroll()
							.overflow_y_scroll()
							.bg(cx.theme().colors().editor_background)
							.text_color(cx.theme().colors().editor_foreground)
							.text_sm()
							.child(self.text.clone()),
					),
			)
			.child(
				div() // Status bar
					.h(px(30.0))
					.border_t_2()
					.border_color(cx.theme().colors().border)
					.bg(cx.theme().colors().status_bar_background)
					.child("Status Bar"),
			)
	}
}

// impl EventEmitter<UIEvent> for FileList {}
