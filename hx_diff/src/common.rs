use gpui::*;

pub static WIDTH: i32 = 1024;
pub static HEIGHT: i32 = 600;

// Setup window helper from Duane Bester https://github.com/duanebester which is based on work
// from Matthias from  // https://github.com/MatthiasGrandl/Loungy
pub fn setup_window(app_width: i32, app_height: i32, cx: &mut AppContext) -> WindowOptions {
	let mut options = WindowOptions::default();

	let width = DevicePixels::from(app_width);
	let height = DevicePixels::from(app_height);
	options.bounds = Some(Bounds::centered(None, Size { width, height }, cx));
	options.titlebar = Some(TitlebarOptions {
		title: Some("HxDiff".into()),
		..Default::default()
	});

	options
}
