mod diff_element;

use self::workspace::{EntryKind, FileEntry, FileSource, ProjectEntryId, Workspace};
use crate::*;
use diff_element::DiffElement;
use git_cli_wrap as git;
use gpui::*;
use similar::{ChangeTag, TextDiff};
use theme::ThemeSettings;

actions!(diff_pane, [PreviousDifference, NextDifference]);

#[derive(Clone, PartialEq, Copy)]
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

#[derive(Clone, Debug)]
pub struct GutterDimensions {
	// pub left_padding: Pixels,
	pub right_padding: Pixels,
	pub width: Pixels,
}

enum MoveDirection {
	Up,
	Down,
}

pub struct DiffStyle {
	text: TextStyle,
}

pub struct DiffPane {
	style: DiffStyle,
	diff_text: SharedString,
	diff_lines: Vec<DiffLine>,
	workspace: Model<Workspace>,
	show_line_numbers: bool,
	scroll_y: f32,
	last_bounds: Option<Bounds<Pixels>>,
	focus_handle: FocusHandle,
	selection: Option<usize>,
}

impl DiffPane {
	pub fn new(
		_hx_diff: WeakView<HxDiff>,
		workspace: Model<Workspace>,
		cx: &mut WindowContext,
	) -> View<DiffPane> {
		let focus_handle = cx.focus_handle();

		cx.on_focus_in(&focus_handle, |_cx| {
			println!("Focus in diff_pane");
		})
		.detach();

		let text_style = TextStyle {
			// TODO: Move into render and save?
			..Default::default()
		};

		let file_list = cx.new_view(|_cx| DiffPane {
			style: DiffStyle { text: text_style },
			diff_text: SharedString::from("Diff content goes here."),
			diff_lines: Vec::new(),
			workspace,
			show_line_numbers: true,
			scroll_y: 0.0,
			focus_handle,
			last_bounds: None,
			selection: None,
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

		// Mid

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
					self.scroll_to(first_change_line, cx);
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
			let chars = line_count.log10().ceil().max(1.);
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

	fn scroll_to(&mut self, index: usize, cx: &mut ViewContext<Self>) {
		self.selection = Some(index);

		let line_height = self.get_line_height(cx);
		let height_in_lines = self.last_bounds.unwrap().size.height / line_height;
		const SCROLL_THRESHOLD: f32 = 0.25;
		let scroll_threshold = height_in_lines * SCROLL_THRESHOLD;

		// Don't scroll if destination line is comfortably visible
		let index_float = index as f32;
		if index_float < self.scroll_y + scroll_threshold {
			self.scroll_y = (index_float - height_in_lines + scroll_threshold).max(0.);
		} else if index_float > self.scroll_y + height_in_lines - scroll_threshold {
			self.scroll_y = (index_float - scroll_threshold).max(0.);
		}
	}

	fn get_line_height(&self, cx: &mut ViewContext<Self>) -> Pixels {
		let settings = ThemeSettings::get_global(cx);
		let font_size = settings.buffer_font_size(cx);
		let line_height = relative(settings.buffer_line_height.value());
		line_height
			.to_pixels(font_size.into(), cx.rem_size())
			.round()
	}

	fn jump_to_next_difference<R>(&mut self, remaining_lines: R, cx: &mut ViewContext<Self>)
	where
		R: Iterator<Item = usize>,
	{
		let mut in_original_diff = true;
		for index in remaining_lines {
			let diff_type = self.diff_lines[index].diff_type;
			if in_original_diff && diff_type == DiffType::Normal {
				in_original_diff = false;
			} else if !in_original_diff && diff_type != DiffType::Normal {
				self.scroll_to(index, cx);
				break;
			}
		}
	}

	fn previous_difference(&mut self, _: &PreviousDifference, cx: &mut ViewContext<Self>) {
		let diff_index = self.selection.unwrap_or(0);
		self.jump_to_next_difference((0..=diff_index).rev(), cx);
	}

	fn next_difference(&mut self, _: &NextDifference, cx: &mut ViewContext<Self>) {
		let diff_index = self.selection.unwrap_or(0);
		self.jump_to_next_difference(diff_index..self.diff_lines.len(), cx);
	}
}

impl FocusableView for DiffPane {
	fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
		// println!("DiffPane::focus_handle");
		self.focus_handle.clone()
	}
}

impl Render for DiffPane {
	fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
		DiffElement::new(cx.view())
	}
}
