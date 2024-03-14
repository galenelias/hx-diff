use crate::*;
use git_cli_wrap::*;
use gpui::*;

#[derive(Debug)]
pub enum FileListEvent {
	OpenedEntry { filename: SharedString },
}

struct ListItem {
	filename: SharedString,
	status: SharedString,
}

impl ListItem {
	fn new(title: String, status: String) -> ListItem {
		ListItem {
			filename: SharedString::from(title),
			status: SharedString::from(status),
		}
	}
}

pub struct FileList {
	status: git_cli_wrap::GitStatus,
	items: Vec<ListItem>,
	hx_diff: WeakView<HxDiff>,
}

impl FileList {
	pub fn new(hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<FileList> {
		let file_list = cx.new_view(|_cx| {
			let status = git_cli_wrap::get_status().expect("Failed to get git status");

			let items = status
				.entries
				.iter()
				.filter(|e| e.unstaged_status != EntryStatus::None)
				.map(|e| ListItem::new(e.path.clone(), e.unstaged_status.to_string()))
				.collect::<Vec<_>>();

			Self {
				status,
				items,
				hx_diff,
			}
		});

		file_list
	}

	fn render_entry(&self, item: &ListItem, cx: &mut ViewContext<Self>) -> Stateful<Div> {
		let filename = item.filename.clone();
		div()
			.flex()
			.flex_row()
			.px_2()
			.hover(|s| s.bg(rgb(0x3a3a3a)))
			.id(SharedString::from(format!(
				"file_list_item_{}",
				&item.filename
			)))
			.on_click(cx.listener(move |_this, _event: &gpui::ClickEvent, cx| {
				cx.emit(FileListEvent::OpenedEntry {
					filename: filename.clone(),
				});
			}))
			.child(div().child(item.filename.clone()).flex_grow().text_sm())
			.child(div().child(item.status.clone()).text_sm())
	}
}

impl Render for FileList {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.min_w_64()
			.gap(px(1.))
			.bg(rgb(0x457b9d))
			.child(div().bg(rgb(0x1d3557)).child("Toolbar"))
			.child(
				uniform_list(cx.view().clone(), "entries", self.items.len(), {
					|this, range, cx| {
						range
							.map(|i| this.render_entry(&this.items[i], cx))
							.collect()
					}
				})
				.w_full(),
			)
	}
}

impl EventEmitter<FileListEvent> for FileList {}
