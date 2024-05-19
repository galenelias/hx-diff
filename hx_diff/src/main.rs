mod common;
mod views;
mod workspace;

use crate::common::{setup_window, HEIGHT, WIDTH};
use crate::workspace::*;
use assets::Assets;
use clap::Parser;
use git_cli_wrap;
use gpui::*;
use settings::{default_settings, Settings, SettingsStore};
use views::*;

actions!(
	app,
	[
		Quit,
		CycleTheme,
		IncreaseFontSize,
		DecreaseFontSize,
		ResetFontSize
	]
);

fn cycle_theme(cx: &mut AppContext) {
	let theme_registry = theme::ThemeRegistry::global(cx);
	let mut theme_settings = theme::ThemeSettings::get_global(cx).clone();
	let all_themes = theme_registry.list_names(true);
	let current_index = all_themes
		.iter()
		.position(|t| t == &theme_settings.active_theme.name)
		.unwrap();
	let new_index = (current_index + 1) % all_themes.len();
	theme_settings.active_theme = theme_registry.get(&all_themes[new_index]).unwrap();
	theme::ThemeSettings::override_global(theme_settings, cx);
	cx.refresh()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// Sub-commmand to run
	pub mode: Option<String>,

	pub arg: Option<String>,

	/// diff options
	#[arg(long = "merge-base")]
	pub merge_base: bool,

	#[arg(long, action)]
	pub cached: bool,

	#[arg(long, action)]
	pub staged: bool,
}

fn main() {
	let args = Args::parse();
	println!("Args = {:?}", args);

	App::new()
		.with_assets(Assets)
		.run(move |cx: &mut AppContext| {
			let mut store = SettingsStore::default();
			store
				.set_default_settings(default_settings().as_ref(), cx)
				.unwrap();
			cx.set_global(store);

			// theme::init(theme::LoadThemes::JustBase, cx); // Only includes "One Dark"
			theme::init(theme::LoadThemes::All(Box::new(Assets)), cx);
			Assets.load_fonts(cx);

			let theme_registry = theme::ThemeRegistry::global(cx);
			let theme_name = "One Dark";

			let mut theme_settings = theme::ThemeSettings::get_global(cx).clone();
			theme_settings.active_theme = theme_registry.get(&theme_name).unwrap();
			theme::ThemeSettings::override_global(theme_settings, cx);

			let options = setup_window(WIDTH, HEIGHT, cx);

			cx.on_action(|_act: &Quit, cx| cx.quit());
			cx.on_action(|_act: &CycleTheme, cx| cycle_theme(cx));
			cx.on_action(|_act: &DecreaseFontSize, cx| {
				theme::adjust_font_size(cx, |size| *size -= px(1.0))
			});
			cx.on_action(|_act: &IncreaseFontSize, cx| {
				theme::adjust_font_size(cx, |size| *size += px(1.0))
			});
			cx.on_action(|_act: &ResetFontSize, cx| theme::reset_font_size(cx));

			cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
			cx.bind_keys([KeyBinding::new("cmd-t", CycleTheme, None)]);
			cx.bind_keys([KeyBinding::new("cmd-+", IncreaseFontSize, None)]);
			cx.bind_keys([KeyBinding::new("cmd--", DecreaseFontSize, None)]);
			cx.bind_keys([KeyBinding::new("cmd-0", ResetFontSize, None)]);

			cx.set_menus(vec![
				Menu {
					name: "",
					items: vec![
						MenuItem::action("Quit", Quit),
						MenuItem::action("Cycle Theme", CycleTheme),
					],
				},
				Menu {
					name: "View",
					items: vec![
						MenuItem::action("Zoom In", IncreaseFontSize),
						MenuItem::action("Decrease Font", DecreaseFontSize),
						MenuItem::action("Reset Zoom", ResetFontSize),
					],
				},
			]);

			let workspace = cx.new_model(|_cx| Workspace::from_args(&args));

			cx.open_window(options, |cx| HxDiff::new(workspace, cx));
			cx.activate(true);
		});
}
