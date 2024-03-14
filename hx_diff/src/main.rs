mod views;

use git_cli_wrap;
use gpui::*;
use views::*;

actions!(app, [Quit]);

fn main() {
	App::new().run(|cx: &mut AppContext| {
		let mut options = WindowOptions::default();
		// options.title = "Hello World".into();
		options.kind = WindowKind::Normal;
		// options.titlebar.unwrap().title = Some(SharedString::from("Hello World"));
		options.focus = true;
		options.bounds = WindowBounds::Fixed(Bounds {
			size: size(px(800.), px(600.)).into(),
			..Default::default()
		});

		cx.on_action(|_act: &Quit, cx| cx.quit());
		cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

		cx.set_menus(vec![Menu {
			name: "",
			items: vec![MenuItem::action("Quit", Quit)],
		}]);

		cx.open_window(options, |cx| cx.new_view(HxDiff::new));
		cx.activate(true);
	});
}
