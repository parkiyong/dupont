use relm4::RelmApp;

mod domain;
mod app;

fn main() {
    let app = RelmApp::new("io.github.parkiyong.dupont");
    app.run_async::<app::App>(());
}
