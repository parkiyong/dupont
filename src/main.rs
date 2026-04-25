mod application;
mod domain;
mod infrastructure;
mod presentation;

fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Dupont wallpaper app");
    presentation::app::AppState::run()
}
