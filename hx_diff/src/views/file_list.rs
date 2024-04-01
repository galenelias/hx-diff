use crate::*;
use git_cli_wrap::*;
use gpui::*;
use hx_diff::{DraggedPanel, PanelPosition};
use std::path::PathBuf;
use theme::ActiveTheme;

const RESIZE_HANDLE_SIZE: Pixels = Pixels(6.);

#[derive(Debug)]
pub enum FileListEvent {
	OpenedEntry { filename: PathBuf, is_staged: bool }, // TODO: flush out to be a full specification of the file/diff
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ListItemType {
	Category,
	Directory,
	File,
}

#[derive(Debug)]
struct ListItem {
	item_type: ListItemType,
	path: PathBuf,
	is_staged: bool,
	label: SharedString,
	status: SharedString,
}

pub struct FileList {
	items: Vec<ListItem>,
	// hx_diff: WeakView<HxDiff>,
	model: Model<FileListModel>,
}

pub struct FileListModel {
	width: Option<Pixels>,
}

impl FileList {
	pub fn new(_hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<FileList> {
		let model = cx.new_model(|_cx| FileListModel { width: None });

		let file_list = cx.new_view(|_cx| {
			let status = git_cli_wrap::get_status().expect("Failed to get git status");

			let mut items = Vec::new();

			let mut process_items = |get_status: fn(&Entry) -> EntryStatus,
			                         is_staged: bool,
			                         category_name: &'static str| {
				let mut has_items = false;
				let mut last_dir = None;

				for entry in status
					.entries
					.iter()
					.filter(|e| get_status(e) != EntryStatus::None)
				{
					let path = &entry.path;

					if !has_items {
						items.push(ListItem {
							item_type: ListItemType::Category,
							path: path.clone().into(),
							is_staged,
							label: SharedString::from(category_name),
							status: "".into(),
						});
						has_items = true;
					}
					let parent_dir = path.parent().expect(&format!(
						"Failed to get parent directory for '{}'",
						entry.path.display()
					));

					if Some(parent_dir) != last_dir {
						last_dir = Some(parent_dir);
						items.push(ListItem {
							item_type: ListItemType::Directory,
							path: path.clone(),
							is_staged,
							label: parent_dir.to_string_lossy().to_string().into(),
							status: "".into(),
						});
					}

					let status = get_status(entry).to_string();
					items.push(ListItem {
						item_type: ListItemType::File,
						path: path.clone(),
						is_staged,
						label: path
							.file_name()
							.unwrap()
							.to_string_lossy()
							.into_owned()
							.into(),
						status: status.into(),
					});
				}
			};

			process_items(
				|e| e.staged_status,
				/*is_staged=*/ true,
				"STAGED - Changes to be committed",
			);
			process_items(
				|e| e.unstaged_status,
				/*is_staged=*/ false,
				"WORKING - Changes not staged for commit",
			);

			Self {
				// status,
				items,
				// hx_diff,
				model,
			}
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
		let is_staged = item.is_staged;

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

		let id = SharedString::from(format!(
			"file_list_item_{}_{}",
			&item.path.to_string_lossy(),
			is_staged
		));

		ui::ListItem::new(index).child(
			div()
				.flex()
				.flex_row()
				.w_full()
				.px_2()
				.text_color(text_color)
				.id(id)
				.on_click(cx.listener(move |_this, _event: &gpui::ClickEvent, cx| {
					if item_type == ListItemType::File {
						cx.emit(FileListEvent::OpenedEntry {
							filename: path.clone(),
							is_staged,
						});
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
