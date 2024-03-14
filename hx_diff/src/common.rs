use gpui::*;

pub static WIDTH: f64 = 800.0;
pub static HEIGHT: f64 = 600.0;

// Setup window helper from Duane Bester https://github.com/duanebester which is based on work
// from Matthias from  // https://github.com/MatthiasGrandl/Loungy
pub fn setup_window(app_width: f64, app_height: f64, cx: &mut AppContext) -> WindowOptions {
	let display_id_maybe = cx.displays().last().map(|d| d.id());
	let bounds_maybe = cx.displays().last().map(|d| d.bounds());
	let bounds = bounds_maybe.unwrap_or(Bounds {
		origin: Point::new(GlobalPixels::from(0.0), GlobalPixels::from(0.0)),
		size: Size {
			width: GlobalPixels::from(1920.0),
			height: GlobalPixels::from(1080.0),
		},
	});

	let mut options = WindowOptions::default();
	let center = bounds.center();

	options.focus = true;
	options.display_id = display_id_maybe;
	let width = GlobalPixels::from(app_width);
	let height = GlobalPixels::from(app_height);
	let x: GlobalPixels = center.x - width / 2.0;
	let y: GlobalPixels = center.y - height / 2.0;

	let bounds: Bounds<GlobalPixels> = Bounds::new(Point { x, y }, Size { width, height });
	options.bounds = WindowBounds::Fixed(bounds);
	options.titlebar = Some(TitlebarOptions::default());
	options.is_movable = true;
	options.kind = WindowKind::Normal;
	options
}
