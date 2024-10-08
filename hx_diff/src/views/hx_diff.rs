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
	_weak_self: WeakView<Self>,
	file_pane: View<FileList>,
	diff_pane: View<DiffPane>,
	_workspace: Model<Workspace>,
}

impl HxDiff {
	pub fn new(workspace: Model<Workspace>, cx: &mut WindowContext) -> View<HxDiff> {
		let hxdiff_view = cx.new_view(|cx| {
			let weak_handle = cx.view().downgrade();

			let file_pane = FileList::new(weak_handle.clone(), workspace.clone(), cx);
			let diff_pane = DiffPane::new(weak_handle.clone(), workspace.clone(), cx);

			cx.subscribe(&file_pane, {
				move |hx_diff, _, event, cx| match event {
					&FileListEvent::OpenedEntry { entry_id } => {
						hx_diff.open_file(entry_id, cx);
					}
				}
			})
			.detach();

			//cx.bind_keys([KeyBinding::new("cmd-r", RefreshFileList, None)]);

			cx.focus_view(&diff_pane);

			HxDiff {
				_weak_self: weak_handle,
				file_pane,
				diff_pane,
				_workspace: workspace.clone(),
			}
		});
		hxdiff_view
	}

	// pub fn _weak_handle(&self) -> WeakView<Self> {
	// 	self._weak_self.clone()
	// }

	fn open_file(&mut self, id: ProjectEntryId, cx: &mut ViewContext<Self>) {
		self.diff_pane.update(cx, |diff_pane, cx| {
			diff_pane.open_diff(id, cx);
		});
	}

	fn refresh_list(&mut self, _: &RefreshFileList, _cx: &mut ViewContext<Self>) {
		println!("HxDiff: Refresh File List!");
	}
}

impl Render for HxDiff {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.size_full()
			.flex()
			.flex_col()
			.text_color(cx.theme().colors().text)
			.on_action(cx.listener(Self::refresh_list))
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

impl FocusableView for HxDiff {
	fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
		self.diff_pane.focus_handle(cx)
	}
}

// impl EventEmitter<UIEvent> for FileList {}
