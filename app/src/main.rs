use relm4::RelmApp;

mod app;
mod messages;
mod widgets;

fn main() {
    let app = RelmApp::new("com.damask.Wallpaper");
    app.run_async::<app::App>(());
}
