use crate::*;
use git_cli_wrap::*;
use gpui::*;

#[derive(Debug)]
pub enum Event {
	OpenedEntry { path: SharedString },
}

#[derive(IntoElement, Clone)]
struct ListItem {
	filename: SharedString,
	status: SharedString,
}

impl RenderOnce for ListItem {
	fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
		div()
			.flex()
			.flex_row()
			.px_2()
			.hover(|s| s.bg(rgb(0x3a3a3a)))
			.id(SharedString::from(format!(
				"file_list_item_{}",
				&self.filename
			)))
			.on_click({
				let filename = self.filename.clone();
				move |_event, _cx| {
					// _cx.emit()
					println!("Clicked on {}", filename);
				}
			})
			.child(div().child(self.filename).flex_grow().text_sm())
			.child(div().child(self.status).text_sm())
	}
}

impl ListItem {
	fn new(title: String, status: String) -> ListItem {
		ListItem {
			filename: SharedString::from(title),
			status: SharedString::from(status),
		}
	}
}

struct State {
	items: Vec<ListItem>,
}

pub struct FileList {
	status: git_cli_wrap::GitStatus,
	list_model: Model<State>,
	list_state: ListState,
	hx_diff: WeakView<HxDiff>,
}

impl FileList {
	pub fn new(hx_diff: WeakView<HxDiff>, cx: &mut WindowContext) -> View<FileList> {
		let file_list = cx.new_view(|cx| {
			let status = git_cli_wrap::get_status().expect("Failed to get git status");

			let items = status
				.entries
				.iter()
				.filter(|e| e.unstaged_status != EntryStatus::None)
				.map(|e| ListItem::new(e.path.clone(), e.unstaged_status.to_string()))
				.collect::<Vec<_>>();

			let items_copy = items.clone();

			let list_state = ListState::new(
				items_copy.len(),
				ListAlignment::Top,
				Pixels(20.0),
				move |idx, _cx| div().child(items_copy[idx].clone()).into_any_element(),
			);
			let list_model = cx.new_model(|_cx| State { items });

			Self {
				status,
				list_model,
				list_state,
				hx_diff,
			}
		});

		cx.subscribe(&file_list, {
			move |_, event, _| match event {
				&Event::OpenedEntry { ref path } => {
					println!("Opened entry: {}", path);
				}
			}
		})
		.detach();

		file_list
	}
}

impl Render for FileList {
	fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.min_w_64()
			.gap(px(1.))
			.bg(rgb(0x457b9d))
			.child(div().bg(rgb(0x1d3557)).child("Toolbar"))
			.child(list(self.list_state.clone()).w_full().h_full())
		// 	div()
		// 		.flex()
		// 		.flex_grow()
		// 		.bg(rgb(0x457b9d))
		// 		.child("FileList\n\n-File 1\n-File 2"),
		// )
	}
}

impl EventEmitter<Event> for FileList {}
