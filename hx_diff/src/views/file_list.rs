use crate::*;
use gpui::*;
use hx_diff::{DraggedPanel, PanelPosition};
use std::path::PathBuf;
use theme::ActiveTheme;

use self::workspace::{EntryKind, ProjectEntryId, Workspace};

const RESIZE_HANDLE_SIZE: Pixels = Pixels(6.);

#[derive(Debug)]
pub enum FileListEvent {
	OpenedEntry { entry_id: ProjectEntryId },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ListItemType {
	Category,
	Directory,
	File,
}

// Test - TODO, delete me

#[derive(Debug)]
struct ListItem {
	item_type: ListItemType,
	entry_id: ProjectEntryId,
	path: PathBuf,
	label: SharedString,
	status: SharedString,
}

pub struct FileList {
	items: Vec<ListItem>,
	// hx_diff: WeakView<HxDiff>,
	model: Model<FileListModel>,
	workspace: Model<Workspace>,
}

impl FileList {
	fn refresh_from_workspace(&mut self, workspace: &Workspace) {
		println!("FileList::refresh_from_workspace()");
		self.items = workspace
			.entries
			.iter()
			.map(|entry| {
				let item_type = match entry.kind {
					EntryKind::Category(_) => ListItemType::Category,
					EntryKind::Directory(_) => ListItemType::Directory,
					EntryKind::File(_) => ListItemType::File,
				};

				let label: SharedString = match entry.kind {
					EntryKind::Category(workspace::CategoryKind::Staged) => {
						"STAGED - Changes to be committed".into()
					}
					EntryKind::Category(workspace::CategoryKind::Working) => {
						"UNSTAGED - Changes not staged for commit".into()
					}
					EntryKind::Category(workspace::CategoryKind::Commit) => {
						"Commit Details Here".into()
					}
					EntryKind::Directory(_) => entry.path.to_string_lossy().into_owned().into(),
					EntryKind::File(ref _name) => entry
						.path
						.file_name()
						.unwrap()
						.to_string_lossy()
						.into_owned()
						.into(),
				};

				// TODO
				let status: SharedString = match entry.kind {
					EntryKind::File(_) => "modified".into(),
					_ => "".into(),
				};

				ListItem {
					item_type,
					entry_id: entry.id,
					path: entry.path.clone(),
					// is_staged: entry.is_staged,
					label: SharedString::from(label),
					status,
				}
			})
			.collect();
	}
}

pub struct FileListModel {
	width: Option<Pixels>,
}

impl FileList {
	pub fn new(
		_hx_diff: WeakView<HxDiff>,
		workspace: Model<Workspace>,
		cx: &mut WindowContext,
	) -> View<FileList> {
		let model = cx.new_model(|_cx| FileListModel { width: None });

		let file_list = cx.new_view(|cx| {
			cx.observe(&workspace, |model: &mut FileList, workspace, cx| {
				model.refresh_from_workspace(workspace.read(cx));
				cx.notify();
			})
			.detach();

			let mut file_list = Self {
				// status,
				items: Vec::new(),
				// hx_diff,
				model,
				workspace,
			};

			file_list.refresh_from_workspace(file_list.workspace.read(cx));

			file_list
		});

		file_list
	}

	pub fn resize_panel(&mut self, size: Option<Pixels>, cx: &mut ViewContext<Self>) {
		self.model.update(cx, |model, cx| {
			model.width = size;
			cx.notify()
		});
	}

	fn render_entry(
		&self,
		item: &ListItem,
		index: usize,
		cx: &mut ViewContext<Self>,
	) -> ui::ListItem {
		let item_type = item.item_type;
		let path = item.path.clone();

		let indent = match item_type {
			ListItemType::Category => 0,
			ListItemType::Directory => 1,
			ListItemType::File => 2,
		};

		let text_color = match item_type {
			ListItemType::Category => cx.theme().colors().text_accent,
			ListItemType::Directory => cx.theme().colors().text_muted,
			ListItemType::File => cx.theme().colors().text,
		};

		let id = item.entry_id;

		ui::ListItem::new(index).child(
			div()
				.flex()
				.flex_row()
				.w_full()
				.px_2()
				.text_color(text_color)
				.id(id.to_usize())
				.on_click(cx.listener(move |_this, _event: &gpui::ClickEvent, cx| {
					if item_type == ListItemType::File {
						cx.emit(FileListEvent::OpenedEntry { entry_id: id });
					}
				}))
				.child(
					div()
						.ml(indent as f32 * px(12.))
						.child(item.label.clone())
						.flex_grow()
						.text_sm(),
				)
				.child(
					div()
						.text_color(cx.theme().colors().text_accent)
						.child(item.status.clone())
						.text_sm(),
				),
		)
	}
}

impl Render for FileList {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		let width = self.model.read(cx).width;

		let handle = div()
			.id("resize-handle")
			.absolute()
			.right(-RESIZE_HANDLE_SIZE / 2.)
			.top(px(0.))
			.h_full()
			.w(RESIZE_HANDLE_SIZE)
			.cursor_col_resize()
			.on_drag(DraggedPanel(PanelPosition::Left), |pane, cx| {
				cx.stop_propagation();
				cx.new_view(|_| pane.clone())
			})
			.occlude();
		div()
			.flex()
			.flex_col()
			.w(width.unwrap_or(px(300.)))
			.gap(px(1.))
			.border_r_1()
			.border_color(cx.theme().colors().border)
			.bg(cx.theme().colors().panel_background)
			.gap(rems(0.3))
			.child(
				div()
					.border_b_1()
					.border_color(cx.theme().colors().border)
					.bg(cx.theme().colors().title_bar_background)
					.pl_3()
					.pt_1()
					.child("Status"),
			)
			.child(
				uniform_list(cx.view().clone(), "entries", self.items.len(), {
					|this, range, cx| {
						range
							.map(|i| this.render_entry(&this.items[i], i, cx))
							.collect()
					}
				})
				.w_full(),
			)
			.child(handle)
	}
}

impl EventEmitter<FileListEvent> for FileList {}
