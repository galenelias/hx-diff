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
		RefreshFileList,
		IncreaseFontSize,
		DecreaseFontSize,
		ResetFontSize
	]
);

fn cycle_theme(cx: &mut App) {
	let theme_registry = theme::ThemeRegistry::global(cx);
	let mut theme_settings = theme::ThemeSettings::get_global(cx).clone();
	let all_themes = theme_registry.list_names();
	let current_index = all_themes
		.iter()
		.position(|t| t == &theme_settings.active_theme.name)
		.unwrap();
	let new_index = (current_index + 1) % all_themes.len();
	theme_settings.active_theme = theme_registry.get(&all_themes[new_index]).unwrap();
	theme::ThemeSettings::override_global(theme_settings, cx);
	cx.refresh_windows()
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

	Application::new()
		.with_assets(Assets)
		.run(move |cx: &mut App| {
			let mut store = SettingsStore::new(cx);
			store
				.set_default_settings(default_settings().as_ref(), cx)
				.unwrap();
			cx.set_global(store);

			// theme::init(theme::LoadThemes::JustBase, cx); // Only includes "One Dark"
			theme::init(theme::LoadThemes::All(Box::new(Assets)), cx);
			Assets.load_fonts(cx).expect("Failed to load fonts");

			let theme_registry = theme::ThemeRegistry::global(cx);
			let theme_name = "One Dark";

			let mut theme_settings = theme::ThemeSettings::get_global(cx).clone();
			theme_settings.active_theme = theme_registry.get(&theme_name).unwrap();
			theme::ThemeSettings::override_global(theme_settings, cx);

			let options = setup_window(WIDTH, HEIGHT, cx);

			cx.on_action(|_act: &Quit, cx| cx.quit());
			cx.on_action(|_act: &CycleTheme, cx| cycle_theme(cx));
			cx.on_action(|_act: &DecreaseFontSize, cx| {
				theme::adjust_buffer_font_size(cx, |size| size - px(1.0))
			});
			cx.on_action(|_act: &IncreaseFontSize, cx| {
				theme::adjust_buffer_font_size(cx, |size| size + px(1.0))
			});
			cx.on_action(|_act: &ResetFontSize, cx| theme::reset_buffer_font_size(cx));

			cx.bind_keys([KeyBinding::new("f7", diff_pane::PreviousDifference, None)]);
			cx.bind_keys([KeyBinding::new("f8", diff_pane::NextDifference, None)]);

			// OS specific key bindings
			if cfg!(target_os = "macos") {
				cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
				cx.bind_keys([KeyBinding::new("cmd-t", CycleTheme, None)]);
				cx.bind_keys([KeyBinding::new("cmd-+", IncreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("cmd-=", IncreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("cmd--", DecreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("cmd-0", ResetFontSize, None)]);
				cx.bind_keys([KeyBinding::new("cmd-r", RefreshFileList, None)]);
			} else if cfg!(target_os = "windows") {
				cx.bind_keys([KeyBinding::new("ctrl-t", CycleTheme, None)]);
				cx.bind_keys([KeyBinding::new("ctrl-+", IncreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("ctrl-=", IncreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("ctrl--", DecreaseFontSize, None)]);
				cx.bind_keys([KeyBinding::new("ctrl-0", ResetFontSize, None)]);
				cx.bind_keys([KeyBinding::new("ctrl-r", RefreshFileList, None)]);
			}

			cx.set_menus(vec![
				Menu {
					name: "".into(),
					items: vec![
						MenuItem::action("Quit", Quit),
						MenuItem::action("Cycle Theme", CycleTheme),
					],
				},
				Menu {
					name: "View".into(),
					items: vec![
						MenuItem::action("Zoom In", IncreaseFontSize),
						MenuItem::action("Decrease Font", DecreaseFontSize),
						MenuItem::action("Reset Zoom", ResetFontSize),
						MenuItem::separator(),
						MenuItem::action("Refresh File List", RefreshFileList),
					],
				},
			]);

			let workspace = cx.new(|_cx| Workspace::from_args(&args));

			cx.open_window(options, |window, cx| HxDiff::new(workspace, window, cx))
				.expect("Failed to create window");
			cx.activate(true);
		});
}
