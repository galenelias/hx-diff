mod common;
mod views;

use crate::common::{setup_window, HEIGHT, WIDTH};
use git_cli_wrap;
use gpui::*;
use views::*;

actions!(app, [Quit]);

fn main() {
	App::new().run(|cx: &mut AppContext| {
		let mut options = setup_window(WIDTH, HEIGHT, cx);

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
