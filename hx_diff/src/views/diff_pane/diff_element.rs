use super::DiffType;
use crate::diff_pane::DiffLine;
use crate::diff_pane::GutterDimensions;
use crate::DiffPane;
use gpui::*;
use settings::Settings;
use std::fmt::Write;
use std::ops::Range;
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

#[derive(Clone, Debug)]
struct ScrollbarLayout {
	hitbox: Hitbox,
	visible_row_range: Range<f32>,
	// visible: bool,
	row_height: Pixels,
	thumb_height: Pixels,
}

impl ScrollbarLayout {
	const BORDER_WIDTH: Pixels = px(1.0);
	const LINE_MARKER_HEIGHT: Pixels = px(2.0);
	const MIN_MARKER_HEIGHT: Pixels = px(5.0);
	const MIN_THUMB_HEIGHT: Pixels = px(20.0);

	fn y_for_row(&self, row: f32) -> Pixels {
		self.hitbox.top() + row * self.row_height
	}

	fn thumb_bounds(&self) -> Bounds<Pixels> {
		let thumb_top = self.y_for_row(self.visible_row_range.start);
		let thumb_bottom = thumb_top + self.thumb_height;
		Bounds::from_corners(
			point(self.hitbox.left(), thumb_top),
			point(self.hitbox.right(), thumb_bottom),
		)
	}

	fn marker_bounds(&self, start: f32, end: f32) -> Bounds<Pixels> {
		let top = self.y_for_row(start);
		let bottom = self.y_for_row(end);
		Bounds::from_corners(
			point(self.hitbox.left(), top),
			point(self.hitbox.right(), bottom),
		)
	}
}

type DiffRegions = Vec<((usize, usize), DiffType)>;

pub struct DiffLayout {
	lines: Vec<(ShapedLine, Hsla)>,
	// gutter_hitbox: Hitbox,
	gutter_dimensions: GutterDimensions,
	scrollbar_layout: Option<ScrollbarLayout>,
	text_hitbox: Hitbox,
	line_height: Pixels,
	line_numbers: Vec<ShapedLine>,
	diff_regions: DiffRegions,
}

impl DiffElement {
	pub(crate) const SCROLLBAR_WIDTH: Pixels = px(13.);

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

	fn compute_diff_regions(&self, diff_lines: &[DiffLine]) -> Vec<((usize, usize), DiffType)> {
		let mut diff_markers = Vec::new();
		let mut last_diff_index = 0;
		let mut last_diff_type = DiffType::Normal;
		for (ix, diff) in diff_lines.iter().enumerate() {
			if diff.diff_type != last_diff_type {
				if last_diff_type != DiffType::Normal {
					diff_markers.push(((last_diff_index, ix), last_diff_type));
				}
				last_diff_index = ix;
				last_diff_type = diff.diff_type;
			}
		}

		if last_diff_type != DiffType::Normal {
			diff_markers.push(((last_diff_index, diff_lines.len()), last_diff_type));
		}

		diff_markers
	}

	fn layout_scrollbar(
		&self,
		total_rows: f32,
		bounds: Bounds<Pixels>,
		scroll_position: gpui::Point<f32>,
		rows_per_page: f32,
		cx: &mut WindowContext,
	) -> Option<ScrollbarLayout> {
		let _show_scrollbars = true; // TODO: contextual
		let visible_row_range = scroll_position.y..scroll_position.y + rows_per_page;

		let track_bounds = Bounds::from_corners(
			point(bounds.right() - DiffElement::SCROLLBAR_WIDTH, bounds.top()),
			point(bounds.right(), bounds.bottom()),
		);

		let scroll_beyond_last_line: f32 = 1.0;
		let total_rows = (total_rows + scroll_beyond_last_line).max(rows_per_page);
		let height = bounds.size.height;
		let px_per_row = height / total_rows;
		let thumb_height = (rows_per_page * px_per_row).max(ScrollbarLayout::MIN_THUMB_HEIGHT);
		let row_height = (height - thumb_height) / (total_rows - rows_per_page).max(0.);

		Some(ScrollbarLayout {
			hitbox: cx.insert_hitbox(track_bounds, false),
			visible_row_range,
			row_height,
			thumb_height,
		})
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

		diff_pane.selection = Some(final_y as usize);

		cx.refresh();

		cx.stop_propagation();
	}

	fn paint_scrollbar(&mut self, layout: &mut DiffLayout, cx: &mut WindowContext) {
		let Some(scrollbar_layout) = layout.scrollbar_layout.as_ref() else {
			return;
		};

		let thumb_bounds = scrollbar_layout.thumb_bounds();

		cx.paint_layer(scrollbar_layout.hitbox.bounds, |cx| {
			cx.paint_quad(quad(
				scrollbar_layout.hitbox.bounds,
				Corners::default(),
				cx.theme().colors().scrollbar_track_background,
				Edges {
					top: Pixels::ZERO,
					right: Pixels::ZERO,
					bottom: Pixels::ZERO,
					left: ScrollbarLayout::BORDER_WIDTH,
				},
				cx.theme().colors().scrollbar_track_border,
			));

			for diff_region in &layout.diff_regions {
				cx.paint_quad(quad(
					scrollbar_layout
						.marker_bounds(diff_region.0 .0 as f32, diff_region.0 .1 as f32),
					Corners::default(),
					if diff_region.1 == DiffType::Added {
						cx.theme().status().created
					} else {
						cx.theme().status().deleted
					},
					Edges {
						top: Pixels::ZERO,
						right: Pixels::ZERO,
						bottom: Pixels::ZERO,
						left: ScrollbarLayout::BORDER_WIDTH,
					},
					cx.theme().colors().scrollbar_thumb_border,
				));
			}

			cx.paint_quad(quad(
				thumb_bounds,
				Corners::default(),
				cx.theme().colors().scrollbar_thumb_background,
				Edges {
					top: Pixels::ZERO,
					right: Pixels::ZERO,
					bottom: Pixels::ZERO,
					left: ScrollbarLayout::BORDER_WIDTH,
				},
				cx.theme().colors().scrollbar_thumb_border,
			));
		});

		let hitbox = scrollbar_layout.hitbox.clone();
		let height_in_lines = hitbox.size.height / layout.line_height;

		cx.on_mouse_event({
			let diff_pane = self.diff_pane.clone();
			let rows_per_page =
				scrollbar_layout.visible_row_range.end - scrollbar_layout.visible_row_range.start;

			move |event: &MouseDownEvent, phase, cx| {
				if phase == DispatchPhase::Capture || !hitbox.is_hovered(cx) {
					return;
				}

				diff_pane.update(cx, |diff_pane, cx| {
					// editor.scroll_manager.set_is_dragging_scrollbar(true, cx);

					let y = event.position.y;

					let is_dragging = diff_pane.scrollbar_drag_state.clone();
					let percentage = (event.position.y - hitbox.top()) / hitbox.size.height;

					// is_dragging.set(Some(5.));
					cx.refresh();
					if y < thumb_bounds.top() || thumb_bounds.bottom() < y {
						// Set the thumb offset as the middle of the thumb
						let thumb_top_offset = thumb_bounds.size.height / 2. / hitbox.size.height;
						is_dragging.set(Some(thumb_top_offset));

						let y = diff_pane.diff_lines.len() as f32 * percentage - rows_per_page / 2.;
						diff_pane.scroll_y = y.clamp(
							0.0,
							diff_pane.diff_lines.len() as f32 - height_in_lines.floor(),
						);
					} else {
						let thumb_top_offset =
							(event.position.y - thumb_bounds.origin.y) / hitbox.size.height;
						is_dragging.set(Some(thumb_top_offset));
					}

					cx.stop_propagation();
				});
			}
		});

		cx.on_mouse_event({
			let diff_pane = self.diff_pane.clone();
			let hitbox = scrollbar_layout.hitbox.clone();

			move |event: &MouseMoveEvent, phase, cx| {
				if phase.capture() {
					return;
				}

				let drag_state = diff_pane.read(cx).scrollbar_drag_state.clone();

				if let Some(drag_state) = drag_state.get().filter(|_| event.dragging()) {
					let percentage =
						(event.position.y - hitbox.top()) / hitbox.size.height - drag_state;

					diff_pane.update(cx, |diff_pane, cx| {
						let y = diff_pane.diff_lines.len() as f32 * percentage;
						diff_pane.scroll_y = y.clamp(
							0.0,
							diff_pane.diff_lines.len() as f32 - height_in_lines.floor(),
						);
						cx.refresh();
					});

					cx.stop_propagation();
				} else {
					drag_state.set(None);
				}
			}
		});

		let is_dragging = self.diff_pane.read(cx).scrollbar_drag_state.clone();
		cx.on_mouse_event(move |_event: &MouseUpEvent, phase, _cx| {
			if phase.bubble() {
				is_dragging.set(None);
			}
		});
	}

	fn paint_mouse_listeners(
		&mut self,
		layout: &mut DiffLayout,
		bounds: Bounds<gpui::Pixels>,
		cx: &mut WindowContext,
	) {
		let line_height = layout.line_height;
		let text_hitbox = layout.text_hitbox.clone();
		let height_in_lines = bounds.size.height / line_height;

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

					diff_pane.update(cx, |diff_pane, cx| {
						match delta {
							ScrollDelta::Lines(_) => (),
							ScrollDelta::Pixels(point) => {
								let y = diff_pane.scroll_y - point.y.0 / line_height.0 as f32;
								diff_pane.scroll_y = y.clamp(
									0.0,
									diff_pane.diff_lines.len() as f32 - height_in_lines.floor(),
								);
								cx.notify();
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
		register_action(view, cx, DiffPane::previous_difference);
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
		let total_rows = diff_lines.len();

		let scrollbar_layout = self.layout_scrollbar(
			total_rows as f32,
			bounds,
			point(0., scroll_y),
			height_in_lines,
			cx,
		);

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

		self.diff_pane.update(cx, |diff_pane, _cx| {
			diff_pane.last_bounds = Some(bounds);
		});

		let diff_regions = self.compute_diff_regions(diff_lines);

		DiffLayout {
			lines,
			// gutter_hitbox,
			gutter_dimensions,
			scrollbar_layout,
			text_hitbox,
			line_height,
			line_numbers,
			diff_regions,
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
		self.paint_mouse_listeners(layout, bounds, cx);

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

		self.paint_scrollbar(layout, cx);
	}
}

impl IntoElement for DiffElement {
	type Element = Self;

	fn into_element(self) -> Self::Element {
		self
	}
}
