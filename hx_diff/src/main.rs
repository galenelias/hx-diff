mod common;
mod views;

use crate::common::{setup_window, HEIGHT, WIDTH};
use assets::Assets;
use git_cli_wrap;
use gpui::*;
use settings::{default_settings, Settings, SettingsStore};
use std::env;
use views::*;

actions!(app, [Quit]);

fn main() {
	App::new().run(|cx: &mut AppContext| {
		let mut store = SettingsStore::default();
		store
			.set_default_settings(default_settings().as_ref(), cx)
			.unwrap();
		cx.set_global(store);

		// theme::init(theme::LoadThemes::JustBase, cx); // Only includes "One Dark"
		theme::init(theme::LoadThemes::All(Box::new(Assets)), cx);

		let theme_registry = theme::ThemeRegistry::global(cx);
		let mut theme_settings = theme::ThemeSettings::get_global(cx).clone();

		let all_themes = theme_registry.list_names(true);
		let args: Vec<String> = env::args().collect();
		let theme_name = if args.len() > 1 {
			println!("Theme argument: {}", args[1]);
			if args[1].chars().all(|c| c.is_ascii_digit()) {
				all_themes[args[1].parse::<usize>().expect("Invalid theme index")].to_string()
			} else {
				args[1].to_string()
			}
		} else {
			"One Dark".to_string()
		};

		theme_settings.active_theme = theme_registry.get(&theme_name).unwrap();
		theme::ThemeSettings::override_global(theme_settings, cx);

		let options = setup_window(WIDTH, HEIGHT, cx);

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
