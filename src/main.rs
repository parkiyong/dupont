use relm4::RelmApp;

mod app;
mod application;
mod domain;
mod infrastructure;
mod presentation;

fn main() {
    let app = RelmApp::new("io.github.parkiyong.dupont");
    app.run_async::<app::window::App>(());
}
