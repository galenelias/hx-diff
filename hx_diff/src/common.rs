use gpui::*;

pub static WIDTH: f64 = 1024.0;
pub static HEIGHT: f64 = 600.0;

// Setup window helper from Duane Bester https://github.com/duanebester which is based on work
// from Matthias from  // https://github.com/MatthiasGrandl/Loungy
pub fn setup_window(app_width: f64, app_height: f64, cx: &mut AppContext) -> WindowOptions {
	let mut options = WindowOptions::default();

	let width = GlobalPixels::from(app_width);
	let height = GlobalPixels::from(app_height);
	options.bounds = Some(Bounds::centered(Size { width, height }, cx));
	options.titlebar = Some(TitlebarOptions {
		title: Some("HxDiff".into()),
		..Default::default()
	});

	options
}
