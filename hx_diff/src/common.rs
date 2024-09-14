use gpui::*;

pub static WIDTH: f32 = 1024.;
pub static HEIGHT: f32 = 600.;

// Setup window helper from Duane Bester https://github.com/duanebester which is based on work
// from Matthias from  // https://github.com/MatthiasGrandl/Loungy
pub fn setup_window(app_width: f32, app_height: f32, cx: &mut AppContext) -> WindowOptions {
	let mut options = WindowOptions::default();

	let width = px(app_width);
	let height = px(app_height);
	options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
		None,
		Size { width, height },
		cx,
	)));
	options.titlebar = Some(TitlebarOptions {
		title: Some("HxDiff".into()),
		..Default::default()
	});

	options
}
