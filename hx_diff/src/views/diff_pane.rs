mod diff_element;

use self::workspace::{EntryKind, FileEntry, FileSource, ProjectEntryId, Workspace};
use crate::*;
use diff_element::DiffElement;
use git_cli_wrap as git;
use gpui::*;
use similar::{ChangeTag, TextDiff};
use theme::ThemeSettings;

#[derive(Clone)]
pub enum DiffType {
	_Header,
	Normal,
	Added,
	Removed,
}

#[derive(Clone)]
pub struct DiffLine {
	pub text: SharedString,
	pub diff_type: DiffType,
	pub _old_index: Option<usize>,
	pub new_index: Option<usize>,
}

#[derive(Clone)]
pub struct GutterDimensions {
	// pub left_padding: Pixels,
	pub right_padding: Pixels,
	pub width: Pixels,
}

pub struct DiffPane {
	diff_text: SharedString,
	diff_lines: Vec<DiffLine>,
	workspace: Model<Workspace>,
	show_line_numbers: bool,
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
			show_line_numbers: true,
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

		self.scroll_y = 0.;

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

				let mut first_change_line = None;

				for change in diff.iter_all_changes() {
					let diff_type = match change.tag() {
						ChangeTag::Delete => DiffType::Removed,
						ChangeTag::Insert => DiffType::Added,
						ChangeTag::Equal => DiffType::Normal,
					};

					if first_change_line.is_none() && change.tag() != ChangeTag::Equal {
						first_change_line = Some(diff_lines.len());
					}

					let text = change;

					diff_lines.push(DiffLine {
						text: text.value().trim_end().to_string().into(),
						diff_type,
						_old_index: change.old_index(),
						new_index: change.new_index(),
					});
				}
				self.diff_lines = diff_lines;

				if let Some(first_change_line) = first_change_line {
					self.scroll_y = (first_change_line as f32 - 4.).max(0.); // TODO; 30%
				}
			}
			EntryKind::Directory(_) => {
				self.diff_text = SharedString::from("Directory diff not supported.");
			}
			EntryKind::Category(_) => {
				self.diff_text = SharedString::from("Category diff not supported.");
			}
		}
	}

	fn get_gutter_dimensions(&self, cx: &AppContext) -> GutterDimensions {
		if self.show_line_numbers {
			let settings = ThemeSettings::get_global(cx);
			let buffer_font = settings.buffer_font.clone();
			let font_size = settings.buffer_font_size(cx);
			let font_id = cx.text_system().resolve_font(&buffer_font);

			let em_advance = cx
				.text_system()
				.advance(font_id, font_size, 'm')
				.unwrap()
				.width;

			let line_count = self.diff_lines.len() as f32;
			let chars = line_count.log10();
			let left_padding = em_advance;
			let right_padding = em_advance;
			GutterDimensions {
				width: chars * em_advance + left_padding + right_padding,
				// left_padding,
				right_padding,
			}
		} else {
			GutterDimensions {
				width: px(0.),
				// left_padding: px(0.),
				right_padding: px(0.),
			}
		}
	}
}

impl Render for DiffPane {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		DiffElement::new(cx.view())
	}
}
