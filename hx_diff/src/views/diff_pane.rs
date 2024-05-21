mod diff_element;

use self::workspace::{EntryKind, FileEntry, FileSource, ProjectEntryId, Workspace};
use crate::*;
use diff_element::DiffElement;
use git_cli_wrap as git;
use gpui::*;
use similar::{ChangeTag, TextDiff};
use theme::ActiveTheme;

enum DiffType {
	Header,
	Normal,
	Added,
	Removed,
}

struct DiffLine {
	text: SharedString,
	diff_type: DiffType,
}

pub struct DiffPane {
	diff_text: SharedString,
	diff_lines: Vec<DiffLine>,
	workspace: Model<Workspace>,
	scroll_y: f32,
}

impl DiffPane {
	pub fn new(
		_hx_diff: WeakView<HxDiff>,
		workspace: Model<Workspace>,
		cx: &mut WindowContext,
	) -> View<DiffPane> {
		let file_list = cx.new_view(|_cx| DiffPane {
			diff_text: SharedString::from("Diff content goes here."),
			diff_lines: Vec::new(),
			workspace,
			scroll_y: 0.0,
		});

		file_list
	}

	pub fn get_file_contents(file_entry: &FileEntry, file_source: &FileSource) -> String {
		match file_source {
			FileSource::Empty => String::new(),
			FileSource::Working => {
				println!("Getting contents: Working");
				std::fs::read_to_string(&file_entry.path).expect("Could not read file.")
			}
			FileSource::Commit(ref sha1)
			| FileSource::Index(ref sha1)
			| FileSource::Head(ref sha1) => {
				println!("Getting contents: Index");
				git::get_file_contents(&file_entry.path, sha1).expect("Failed to get Index content")
			}
		}
	}

	pub fn open_diff(&mut self, id: ProjectEntryId, cx: &mut ViewContext<Self>) {
		let entry = self
			.workspace
			.read(cx)
			.get_entry(id)
			.expect("Entry not found.");

		match entry.kind {
			EntryKind::File(ref file_entry) => {
				let left_contents =
					DiffPane::get_file_contents(file_entry, &file_entry.left_source);

				let right_contents =
					DiffPane::get_file_contents(file_entry, &file_entry.right_source);

				let diff = TextDiff::from_lines(&left_contents, &right_contents);
				let mut diff_lines = Vec::new();
				// for group in diff.grouped_ops(3) {
				// 	diff_lines.push(DiffLine {
				// 		text: "Diff Group".into(),
				// 		diff_type: DiffType::Header,
				// 	});

				// 	for op in group {
				// 		for change in diff.iter_changes(&op) {
				// 			let diff_type = match change.tag() {
				// 				ChangeTag::Delete => DiffType::Removed,
				// 				ChangeTag::Insert => DiffType::Added,
				// 				ChangeTag::Equal => DiffType::Normal,
				// 			};

				// 			let text = change;

				// 			diff_lines.push(DiffLine {
				// 				text: text.value().trim_end().to_string().into(),
				// 				diff_type,
				// 			});
				// 		}
				// 	}
				// }
				for change in diff.iter_all_changes() {
					let diff_type = match change.tag() {
						ChangeTag::Delete => DiffType::Removed,
						ChangeTag::Insert => DiffType::Added,
						ChangeTag::Equal => DiffType::Normal,
					};

					let text = change;

					diff_lines.push(DiffLine {
						text: text.value().trim_end().to_string().into(),
						diff_type,
					});
				}
				self.diff_lines = diff_lines;
			}
			EntryKind::Directory(_) => {
				self.diff_text = SharedString::from("Directory diff not supported.");
			}
			EntryKind::Category(_) => {
				self.diff_text = SharedString::from("Category diff not supported.");
			}
		}
	}

	fn render_diff_line(&self, item: &DiffLine, cx: &mut ViewContext<Self>) -> Div {
		let color = match item.diff_type {
			DiffType::Header => opaque_grey(0.5, 1.0),
			DiffType::Normal => cx.theme().colors().editor_foreground,
			DiffType::Added => cx.theme().status().created,
			DiffType::Removed => cx.theme().status().deleted,
		};

		let background_color = match item.diff_type {
			DiffType::Header => cx.theme().colors().editor_background,
			DiffType::Normal => cx.theme().colors().editor_background,
			DiffType::Added => cx.theme().status().created_background,
			DiffType::Removed => cx.theme().status().deleted_background,
		};

		div()
			.flex()
			.flex_row()
			.flex_grow()
			.w_full()
			.bg(background_color)
			.pl_3()
			// .border_t_width(px(3.))
			// .border_color(cx.theme().colors().editor_background)
			// .when_some(border, |el, border| {
			// 	el.border_t_width(border)
			// 		.border_color(cx.theme().colors().border)
			// })
			.hover(|s| s.bg(cx.theme().colors().element_hover))
			.child(
				div()
					.flex()
					.flex_grow()
					.flex_nowrap()
					.overflow_x_hidden()
					.text_color(color)
					.child(item.text.clone()),
			)
	}
}

impl Render for DiffPane {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		DiffElement::new(cx.view())

		// let settings = ThemeSettings::get_global(cx);
		// div()
		// 	.flex()
		// 	.flex_col()
		// 	.flex_1()
		// 	.bg(cx.theme().colors().editor_background)
		// 	.text_size(settings.buffer_font_size(cx))
		// 	.font_family(settings.buffer_font.family.clone())
		// 	.child(uniform_list(
		// 		cx.view().clone(),
		// 		"entries",
		// 		self.diff_lines.len(),
		// 		{
		// 			|this, range, cx| {
		// 				range
		// 					.map(|i| this.render_diff_line(&this.diff_lines[i], cx))
		// 					.collect()
		// 			}
		// 		},
		// 	))
	}
}
