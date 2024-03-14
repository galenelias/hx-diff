use gpui::*;

use git_cli_wrap;

actions!(app, [Quit]);

struct HxDiff {
    text: SharedString,
}

impl Render for HxDiff {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            // .size(Length::Definite(Pixels(300.0).into()))
            .flex()
            .flex_col()
            .child(
                div() // main status bar
                    .flex_grow()
                    .flex()
                    .flex_row()
                    .child(div().bg(rgb(0x457b9d)).min_w(px(150.0)).child("File List"))
                    .child(div().flex_grow().bg(rgb(0xa8dadc)).child(self.text.clone())),
            )
            .child(
                div() // Status bar
                    .min_h(px(30.0))
                    .bg(rgb(0x1d3557))
                    .child("Status Bar"),
            )
    }
}

fn main() {
    let status = git_cli_wrap::get_status().expect("Failed to get git status");

    App::new().run(|cx: &mut AppContext| {
        // cx.activate(true);

        let mut options = WindowOptions::default();
        // options.title = "Hello World".into();
        options.kind = WindowKind::Normal;
        // options.titlebar.unwrap().title = Some(SharedString::from("Hello World"));
        options.focus = true;
        options.bounds = WindowBounds::Fixed(Bounds {
            size: size(px(800.), px(600.)).into(),
            ..Default::default()
        });

        cx.on_action(|act: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.set_menus(vec![Menu {
            name: "",
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        cx.open_window(options, |cx| {
            cx.new_view(|_cx| HxDiff {
                text: SharedString::from(status.branch_head),
            })
        });

        cx.activate(true);
    });
}
