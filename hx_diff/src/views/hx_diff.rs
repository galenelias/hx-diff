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
	_weak_self: WeakEntity<Self>,
	file_pane: Entity<FileList>,
	diff_pane: Entity<DiffPane>,
	_workspace: Entity<Workspace>,
}

impl HxDiff {
	pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut App) -> Entity<HxDiff> {
		let hxdiff_view = cx.new(|cx| {
			let weak_handle = cx.entity().downgrade();

			let file_pane = FileList::new(weak_handle.clone(), workspace.clone(), cx);
			let diff_pane = DiffPane::new(weak_handle.clone(), workspace.clone(), window, cx);

			cx.subscribe_in(&file_pane, window, {
				move |hx_diff, _, event, window, cx| match event {
					&FileListEvent::OpenedEntry { entry_id } => {
						hx_diff.open_file(entry_id, window, cx);
					}
				}
			})
			.detach();

			//cx.bind_keys([KeyBinding::new("cmd-r", RefreshFileList, None)]);

			cx.focus_view(&diff_pane, window);

			HxDiff {
				_weak_self: weak_handle,
				file_pane,
				diff_pane,
				_workspace: workspace.clone(),
			}
		});
		hxdiff_view
	}

	// pub fn _weak_handle(&self) -> WeakEntity<Self> {
	// 	self._weak_self.clone()
	// }

	fn open_file(&mut self, id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>) {
		self.diff_pane.update(cx, |diff_pane, cx| {
			diff_pane.open_diff(id, window, cx);
		});
	}

	fn refresh_list(&mut self, _: &RefreshFileList, _window: &mut Window, _cx: &mut Context<Self>) {
		println!("HxDiff: Refresh File List!");
	}
}

impl Render for HxDiff {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
					.on_drag_move(cx.listener(
						move |this, e: &DragMoveEvent<DraggedPanel>, _window, cx| {
							match e.drag(cx).0 {
								PanelPosition::Left => {
									let size = /*this.bounds.left() +*/ e.event.position.x;
									this.file_pane.update(cx, |file_pane, cx| {
										file_pane.resize_panel(Some(size), cx);
									});
								}
							}
						},
					))
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

impl Focusable for HxDiff {
	fn focus_handle(&self, cx: &App) -> FocusHandle {
		self.diff_pane.focus_handle(cx)
	}
}

// impl EventEmitter<UIEvent> for FileList {}
