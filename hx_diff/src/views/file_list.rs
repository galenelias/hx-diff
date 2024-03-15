use crate::*;
use git_cli_wrap::*;
use gpui::*;
use hx_diff::{DraggedPanel, PanelPosition};
use theme::ActiveTheme;

const RESIZE_HANDLE_SIZE: Pixels = Pixels(6.);

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
	model: Model<FileListModel>,
}

pub struct FileListModel {
	width: Option<Pixels>,
}

impl FileList {
	pub fn new(hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<FileList> {
		let model = cx.new_model(|_cx| FileListModel { width: None });

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

	fn render_entry(&self, item: &ListItem, cx: &mut ViewContext<Self>) -> Stateful<Div> {
		let filename = item.filename.clone();
		div()
			.flex()
			.flex_row()
			.w_full()
			.px_2()
			.hover(|s| s.bg(cx.theme().colors().element_hover))
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
			.child(
				div()
					.text_color(cx.theme().colors().text_accent)
					.child(item.status.clone())
					.text_sm(),
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
							.map(|i| this.render_entry(&this.items[i], cx))
							.collect()
					}
				})
				.w_full(),
			)
			.child(handle)
	}
}

impl EventEmitter<FileListEvent> for FileList {}
