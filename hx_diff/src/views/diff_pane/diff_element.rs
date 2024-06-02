use crate::DiffPane;
use gpui::*;
use settings::Settings;
use theme::{ActiveTheme, ThemeSettings};

use super::DiffType;

// Custom Element for custom rendering of diffs
pub struct DiffElement {
	diff_pane: View<DiffPane>,
}

pub struct DiffLayout {
	lines: Vec<(ShapedLine, Hsla)>,
	hitbox: Hitbox,
	line_height: Pixels,
}

impl DiffElement {
	pub fn new(diff_pane: &View<DiffPane>) -> Self {
		Self {
			diff_pane: diff_pane.clone(),
		}
	}

	fn paint_mouse_listeners(&mut self, layout: &mut DiffLayout, cx: &mut WindowContext) {
		let diff_pane = self.diff_pane.clone();
		let line_height = layout.line_height;

		cx.on_mouse_event({
			let mut delta = ScrollDelta::default();
			let hitbox = layout.hitbox.clone();

			move |event: &ScrollWheelEvent, phase, cx| {
				if phase == DispatchPhase::Bubble && hitbox.is_hovered(cx) {
					delta = delta.coalesce(event.delta);

					diff_pane.update(cx, |diff_pane, _cx| {
						match delta {
							ScrollDelta::Lines(_) => (),
							ScrollDelta::Pixels(point) => {
								let y = diff_pane.scroll_y - point.y.0 / line_height.0 as f32;
								diff_pane.scroll_y =
									y.clamp(0.0, diff_pane.diff_lines.len() as f32);
								_cx.notify();
							}
						};
					});
					cx.stop_propagation();
				}
			}
		});
	}
}

impl Element for DiffElement {
	type RequestLayoutState = ();
	type PrepaintState = DiffLayout;

	fn request_layout(&mut self, cx: &mut WindowContext) -> (gpui::LayoutId, ()) {
		let mut style = Style::default();
		style.size.width = relative(1.).into();
		style.size.height = relative(1.).into();
		let layout_id = cx.request_layout(&style, None);

		(layout_id, ())
	}

	fn prepaint(
		&mut self,
		bounds: Bounds<Pixels>,
		_: &mut Self::RequestLayoutState,
		cx: &mut WindowContext,
	) -> Self::PrepaintState {
		let mut lines = Vec::new();
		let settings = ThemeSettings::get_global(cx);
		let buffer_font = settings.buffer_font.clone();
		let font_size = settings.buffer_font_size(cx);
		let line_height = relative(settings.buffer_line_height.value());
		let line_height = line_height
			.to_pixels(font_size.into(), cx.rem_size())
			.round();

		let hitbox = cx.insert_hitbox(bounds, false);

		let diff_lines = &self.diff_pane.read(cx).diff_lines;
		let scroll_y = self.diff_pane.read(cx).scroll_y;

		let start_row = scroll_y as usize;
		let height_in_lines = bounds.size.height / line_height;
		let max_row = std::cmp::min(
			(scroll_y + height_in_lines).ceil() as usize,
			diff_lines.len(),
		);

		for i in start_row..max_row {
			let diff_line = &diff_lines[i];

			let color = match diff_line.diff_type {
				DiffType::Header => opaque_grey(0.5, 1.0),
				DiffType::Normal => cx.theme().colors().editor_foreground,
				DiffType::Added => cx.theme().status().created,
				DiffType::Removed => cx.theme().status().deleted,
			};

			let background_color = match diff_line.diff_type {
				DiffType::Header => cx.theme().colors().editor_background,
				DiffType::Normal => cx.theme().colors().editor_background,
				DiffType::Added => cx.theme().status().created_background,
				DiffType::Removed => cx.theme().status().deleted_background,
			};
			let run = TextRun {
				len: diff_line.text.len(),
				font: buffer_font.clone(),
				color,
				background_color: None, //Some(background_color),
				underline: None,
				strikethrough: None,
			};
			let shaped_line = cx
				.text_system()
				.shape_line(diff_line.text.clone(), font_size, &[run])
				.unwrap();
			lines.push((shaped_line, background_color))
		}

		DiffLayout {
			lines,
			hitbox,
			line_height,
		}
	}

	fn paint(
		&mut self,
		bounds: Bounds<gpui::Pixels>,
		_: &mut Self::RequestLayoutState,
		layout: &mut Self::PrepaintState,
		cx: &mut WindowContext,
	) {
		self.paint_mouse_listeners(layout, cx);

		// cx.with_text_style(Some(text_style), |cx| {
		cx.paint_quad(fill(bounds, cx.theme().colors().editor_background));

		let scroll_y = self.diff_pane.read(cx).scroll_y;
		let scroll_top = scroll_y * layout.line_height;

		for (i, line) in layout.lines.iter().enumerate() {
			let y = i as f32 * layout.line_height - (scroll_top % layout.line_height);

			let origin = bounds.origin + point(px(0.0), y);
			let size = size(bounds.size.width, layout.line_height);
			cx.paint_quad(fill(Bounds { origin, size }, line.1));

			line.0.paint(origin, layout.line_height, cx);
		}
		// for (ix, line) in layout.lines.iter().enumerate() {
		// 	// let y = ix as f32 * line.height;
		// 	let line_origin = bounds.origin + point(px(0.0), ix as f32 * layout.line_height);

		// 	line.paint(line_origin, layout.line_height, cx);
		// }
		// });
	}
}

impl IntoElement for DiffElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
