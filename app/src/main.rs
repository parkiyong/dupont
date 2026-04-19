use relm4::RelmApp;

mod app;
mod config;
mod messages;
mod widgets;

fn main() {
    let app = RelmApp::new("io.github.parkiyong.dupont");
    app.run_async::<app::App>(());
}
