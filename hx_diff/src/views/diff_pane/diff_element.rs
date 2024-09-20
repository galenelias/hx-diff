use super::DiffType;
use crate::diff_pane;
use crate::diff_pane::DiffLine;
use crate::diff_pane::GutterDimensions;
use crate::DiffPane;
use gpui::*;
use settings::Settings;
use std::fmt::Write;
use theme::{ActiveTheme, ThemeSettings};

pub fn register_action<T: Action>(
	view: &View<DiffPane>,
	cx: &mut WindowContext,
	listener: impl Fn(&mut DiffPane, &T, &mut ViewContext<DiffPane>) + 'static,
) {
	let view = view.clone();
	cx.on_action(std::any::TypeId::of::<T>(), move |action, phase, cx| {
		let action = action.downcast_ref().unwrap();
		if phase == DispatchPhase::Bubble {
			view.update(cx, |editor, cx| {
				listener(editor, action, cx);
			})
		}
	})
}

// Custom Element for custom rendering of diffs
pub struct DiffElement {
	diff_pane: View<DiffPane>,
}

pub struct DiffLayout {
	lines: Vec<(ShapedLine, Hsla)>,
	// gutter_hitbox: Hitbox,
	gutter_dimensions: GutterDimensions,
	text_hitbox: Hitbox,
	line_height: Pixels,
	line_numbers: Vec<ShapedLine>,
}

impl DiffElement {
	pub fn new(diff_pane: &View<DiffPane>) -> Self {
		Self {
			diff_pane: diff_pane.clone(),
		}
	}

	fn layout_line_numbers(
		&self,
		rows: std::ops::Range<usize>,
		diff_lines: &[DiffLine],
		cx: &mut WindowContext,
	) -> Vec<ShapedLine> {
		let diff_pane = self.diff_pane.clone();
		let show_line_numbers = diff_pane.read(cx).show_line_numbers;
		let selection = diff_pane.read(cx).selection;

		if show_line_numbers {
			let mut scratch_string = String::new();
			let settings = ThemeSettings::get_global(cx);
			let buffer_font = settings.buffer_font.clone();
			let base_color = cx.theme().colors().editor_line_number;
			let active_color = cx.theme().colors().editor_active_line_number;
			let active_bg_color = cx.theme().colors().editor_active_line_background;
			let font_size = settings.buffer_font_size(cx);

			rows.map(|ix| {
				let diff_line = &diff_lines[ix];
				scratch_string.clear();
				if let Some(new_index) = diff_line.new_index {
					write!(&mut scratch_string, "{}", new_index).unwrap();
				}

				let (color, background_color) = if Some(ix) == selection {
					(active_color, Some(active_bg_color))
				} else {
					(base_color, None)
				};

				let run = TextRun {
					len: scratch_string.len(),
					font: buffer_font.clone(),
					color,
					background_color,
					underline: None,
					strikethrough: None,
				};
				cx.text_system()
					.shape_line(scratch_string.clone().into(), font_size, &[run])
					.unwrap()
			})
			.collect()
		} else {
			Vec::new()
		}
	}

	fn mouse_left_down(
		diff_pane: &mut DiffPane,
		event: &MouseDownEvent,
		text_hitbox: &Hitbox,
		line_height: Pixels,
		cx: &mut ViewContext<DiffPane>,
	) {
		if cx.default_prevented() {
			return;
		}

		if !text_hitbox.is_hovered(cx) {
			return;
		}

		let line_height = line_height;
		let click_y = (event.position.y - text_hitbox.top()) / line_height;
		let final_y = click_y + diff_pane.scroll_y;

		println!("Clicked on line: {}", final_y as usize);

		diff_pane.selection = Some(final_y as usize);

		cx.stop_propagation();
	}

	fn paint_mouse_listeners(&mut self, layout: &mut DiffLayout, cx: &mut WindowContext) {
		let line_height = layout.line_height;
		let text_hitbox = layout.text_hitbox.clone();

		cx.on_mouse_event({
			let diff_pane = self.diff_pane.clone();

			move |event: &MouseDownEvent, phase, cx| {
				if phase == DispatchPhase::Bubble {
					match event.button {
						MouseButton::Left => diff_pane.update(cx, |diff_pane, cx| {
							Self::mouse_left_down(diff_pane, event, &text_hitbox, line_height, cx);
						}),
						_ => (),
					}
				}
			}
		});

		cx.on_mouse_event({
			let diff_pane = self.diff_pane.clone();

			let mut delta = ScrollDelta::default();
			let hitbox = layout.text_hitbox.clone();

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

	fn register_actions(&self, cx: &mut WindowContext) {
		let view = &self.diff_pane;

		// TODO: Maybe move this out to HxDiff.
		register_action(view, cx, DiffPane::next_difference);

		cx.bind_keys([KeyBinding::new("f8", diff_pane::NextDifference, None)]);
	}
}

impl Element for DiffElement {
	type RequestLayoutState = ();
	type PrepaintState = DiffLayout;

	fn id(&self) -> Option<ElementId> {
		None
	}

	fn request_layout(
		&mut self,
		_: Option<&GlobalElementId>,
		cx: &mut WindowContext,
	) -> (gpui::LayoutId, ()) {
		let mut style = Style::default();
		style.size.width = relative(1.).into();
		style.size.height = relative(1.).into();
		let layout_id = cx.request_layout(style, None);

		(layout_id, ())
	}

	fn prepaint(
		&mut self,
		_: Option<&GlobalElementId>,
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

		let diff_lines = &self.diff_pane.read(cx).diff_lines.clone(); // TODO: How to not clone?
		let scroll_y = self.diff_pane.read(cx).scroll_y;
		let selection = self.diff_pane.read(cx).selection;

		let focus_handle = self.diff_pane.focus_handle(cx);
		cx.set_focus_handle(&focus_handle);

		let start_row = scroll_y as usize;
		let height_in_lines = bounds.size.height / line_height;
		let max_row = std::cmp::min(
			(scroll_y + height_in_lines).ceil() as usize,
			diff_lines.len(),
		);

		let gutter_dimensions = self.diff_pane.read(cx).get_gutter_dimensions(cx);
		let gutter_bounds = Bounds {
			origin: bounds.origin,
			size: size(gutter_dimensions.width, bounds.size.height),
		};
		// let gutter_hitbox = cx.insert_hitbox(gutter_bounds, false);
		let text_hitbox = cx.insert_hitbox(
			Bounds {
				origin: gutter_bounds.upper_right(),
				size: size(
					bounds.size.width - gutter_dimensions.width,
					bounds.size.height,
				),
			},
			false,
		);

		let line_numbers = self.layout_line_numbers(start_row..max_row, diff_lines, cx);

		for i in start_row..max_row {
			let diff_line = &diff_lines[i];
			let is_active = Some(i) == selection;

			let color = match diff_line.diff_type {
				DiffType::_Header => opaque_grey(0.5, 1.0),
				DiffType::Normal => cx.theme().colors().editor_foreground,
				DiffType::Added => cx.theme().status().created,
				DiffType::Removed => cx.theme().status().deleted,
			};

			let background_color = match (is_active, diff_line.diff_type) {
				(_, DiffType::_Header) => cx.theme().colors().editor_background,
				(false, DiffType::Normal) => cx.theme().colors().editor_background,
				(true, DiffType::Normal) => cx.theme().colors().editor_active_line_background,
				(_, DiffType::Added) => cx.theme().status().created_background,
				(_, DiffType::Removed) => cx.theme().status().deleted_background,
			};
			let run = TextRun {
				len: diff_line.text.len(),
				font: buffer_font.clone(),
				color,
				background_color: None,
				underline: None,
				strikethrough: None,
			};
			let shaped_line = cx
				.text_system()
				.shape_line(diff_line.text.clone(), font_size, &[run])
				.unwrap();
			lines.push((shaped_line, background_color))
		}

		self.diff_pane.update(cx, |diff_pane, cx| {
			diff_pane.last_bounds = Some(bounds);
		});

		DiffLayout {
			lines,
			// gutter_hitbox,
			gutter_dimensions,
			text_hitbox,
			line_height,
			line_numbers,
		}
	}

	fn paint(
		&mut self,
		_: Option<&GlobalElementId>,
		bounds: Bounds<gpui::Pixels>,
		_: &mut Self::RequestLayoutState,
		layout: &mut Self::PrepaintState,
		cx: &mut WindowContext,
	) {
		self.paint_mouse_listeners(layout, cx);

		// I guess GPUI registers action on every 'frame'... weird.
		self.register_actions(cx);

		// cx.with_text_style(Some(text_style), |cx| {
		cx.paint_quad(fill(bounds, cx.theme().colors().editor_background));

		let scroll_y = self.diff_pane.read(cx).scroll_y;
		let scroll_top = scroll_y * layout.line_height;
		let selection = self.diff_pane.read(cx).selection;
		let active_line_background = cx.theme().colors().editor_active_line_background;

		for (i, line_number) in layout.line_numbers.iter().enumerate() {
			let y = i as f32 * layout.line_height - (scroll_top % layout.line_height);
			let row = scroll_y as usize + i;

			if Some(row) == selection {
				let bounds = Bounds {
					origin: bounds.origin + point(px(0.), y),
					size: size(layout.gutter_dimensions.width, layout.line_height),
				};
				cx.paint_quad(fill(bounds, active_line_background));
			}

			let origin = bounds.origin
				+ point(
					layout.gutter_dimensions.width
						- layout.gutter_dimensions.right_padding
						- line_number.width,
					y,
				);
			line_number
				.paint(origin, layout.line_height, cx)
				.expect("Failed to paint line number");
		}

		for (i, line) in layout.lines.iter().enumerate() {
			let y = i as f32 * layout.line_height - (scroll_top % layout.line_height);

			let origin = bounds.origin + point(layout.gutter_dimensions.width, y);
			let size = size(bounds.size.width, layout.line_height);
			cx.paint_quad(fill(Bounds { origin, size }, line.1));

			line.0
				.paint(origin, layout.line_height, cx)
				.expect("Failed to paint line");
		}
	}
}

impl IntoElement for DiffElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
