use crate::*;
use gpui::prelude::*;
use gpui::*;
use theme::ActiveTheme;

use crate::workspace::*;

#[derive(Clone)]
pub enum PanelPosition {
	Left,
	// Right,
}

#[derive(Clone, Render)]
pub struct DraggedPanel(pub PanelPosition);

pub struct HxDiff {
	weak_self: WeakView<Self>,
	file_pane: View<FileList>,
	diff_pane: View<DiffPane>,
	workspace: Model<Workspace>,
}

impl HxDiff {
	pub fn new(cx: &mut ViewContext<Self>) -> HxDiff {
		let weak_handle = cx.view().downgrade();

		let workspace = cx.new_model(|_cx| Workspace::for_git_status());

		let file_pane = FileList::new(weak_handle.clone(), workspace.clone(), cx);
		let diff_pane = DiffPane::new(weak_handle.clone(), cx);

		cx.subscribe(&file_pane, {
			move |hx_diff, _, event, cx| match event {
				&FileListEvent::OpenedEntry {
					ref filename,
					is_staged,
				} => {
					hx_diff.open_file(filename, is_staged, cx);
				}
			}
		})
		.detach();

		HxDiff {
			weak_self: weak_handle,
			file_pane,
			diff_pane,
			workspace: workspace.clone(),
		}
	}

	pub fn _weak_handle(&self) -> WeakView<Self> {
		self.weak_self.clone()
	}

	fn open_file(
		&mut self,
		filename: &std::path::Path,
		is_staged: bool,
		cx: &mut ViewContext<Self>,
	) {
		self.diff_pane.update(cx, |diff_pane, cx| {
			diff_pane.open_diff(filename, is_staged, cx);
		});
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
							match e.drag(cx).0 {
								PanelPosition::Left => {
									let size = /*this.bounds.left() +*/ e.event.position.x;
									this.file_pane.update(cx, |file_pane, cx| {
										file_pane.resize_panel(Some(size), cx);
									});
								}
							}
						}),
					)
					.child(self.file_pane.clone())
					.child(self.diff_pane.clone()),
			)
		// .child(
		// 	div() // Status bar - Nothing useful here yet
		// 		.h(px(30.0))
		// 		.border_t_2()
		// 		.border_color(cx.theme().colors().border)
		// 		.bg(cx.theme().colors().status_bar_background)
		// 		.child("Status Bar"),

		// )
	}
}

// impl EventEmitter<UIEvent> for FileList {}
