use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use iced::{
    widget::{button, container, text, Column, Row},
    Element, Length, Task, Theme,
};
use iced_widget::Space;

use crate::application::dto::SettingsDto;
use crate::domain::{BingSource, Cache, SpotlightSource};
use crate::infrastructure::desktop::create_desktop_backend;
use crate::infrastructure::persistence::ConfigRepo;

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    SourceSelected(Source),
    SettingsOpen,
    SettingsClose,
    WallpaperFetched(Result<(String, String, String, PathBuf), String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Bing,
    Spotlight,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Source {
    fn all() -> Vec<Source> {
        vec![Source::Bing, Source::Spotlight]
    }

    fn label(&self) -> &'static str {
        match self {
            Source::Bing => "Bing Wallpaper of the Day",
            Source::Spotlight => "Microsoft Spotlight",
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    cache: Arc<Mutex<Cache>>,
    settings_repo: ConfigRepo,
    current_wallpaper: Option<(String, String, String, PathBuf)>,
    loading: bool,
    show_settings: bool,
    selected_source: Source,
    settings: SettingsDto,
}

impl AppState {
    pub fn new() -> Self {
        let cache =
            Cache::new(crate::domain::CacheConfig::default()).expect("Failed to initialize cache");
        let settings_repo = ConfigRepo::new();
        let settings = settings_repo.load();

        let current_wallpaper = find_latest_cached_image(&cache);

        Self {
            cache: Arc::new(Mutex::new(cache)),
            settings_repo,
            current_wallpaper,
            loading: false,
            show_settings: false,
            selected_source: Source::Bing,
            settings,
        }
    }

    pub fn run() -> iced::Result {
        iced::application(AppState::new, update, view)
            .theme(|_: &AppState| Theme::Dark)
            .run()
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Refresh => {
            if state.loading {
                return Task::none();
            }
            state.loading = true;

            let cache = state.cache.clone();
            let settings = state.settings.clone();
            let source = state.selected_source;

            Task::perform(
                async move {
                    let src: Box<dyn crate::domain::Source> = match source {
                        Source::Bing => Box::new(BingSource::with_market(settings.bing_market)),
                        Source::Spotlight => {
                            Box::new(SpotlightSource::with_locale(settings.spotlight_locale))
                        }
                    };

                    let wallpaper = match src.fetch().await {
                        Ok(w) => w,
                        Err(e) => return Err(e.to_string()),
                    };

                    let cache_path = {
                        let mut guard = cache.lock().await;
                        match guard.get_or_download(&wallpaper).await {
                            Ok(p) => p,
                            Err(e) => return Err(e.to_string()),
                        }
                    };

                    Ok((
                        wallpaper.title,
                        wallpaper.description,
                        wallpaper.attribution,
                        cache_path,
                    ))
                },
                Message::WallpaperFetched,
            )
        }

        Message::SourceSelected(source) => {
            state.selected_source = source;
            update(state, Message::Refresh)
        }

        Message::SettingsOpen => {
            state.show_settings = true;
            Task::none()
        }

        Message::SettingsClose => {
            state.show_settings = false;
            Task::none()
        }

        Message::WallpaperFetched(result) => {
            state.loading = false;

            match result {
                Ok((title, description, attribution, cache_path)) => {
                    state.current_wallpaper =
                        Some((title, description, attribution, cache_path.clone()));

                    if let Ok(backend) = create_desktop_backend() {
                        if let Err(e) = backend.set_wallpaper(&cache_path) {
                            log::error!("Failed to set wallpaper: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch wallpaper: {}", e);
                }
            }
            Task::none()
        }
    }
}

fn view(state: &AppState) -> Element<Message> {
    if state.show_settings {
        return settings_view(state);
    }

    let controls = build_controls(state);

    match &state.current_wallpaper {
        Some((_, _, _, path)) => {
            let image = iced_widget::Image::new(iced_widget::image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced_core::ContentFit::Cover);

            Column::with_children([
                container(image).height(Length::Fill).into(),
                controls,
            ])
            .spacing(0)
            .padding(0)
            .into()
        }
        None => controls,
    }
}

fn build_controls(state: &AppState) -> Element<Message> {
    let is_loading = state.loading;
    let selected = state.selected_source;
    
    let (title_text, desc_text) = state.current_wallpaper.as_ref()
        .map(|(t, d, _, _)| (t.clone(), d.clone()))
        .unwrap_or((String::new(), String::new()));
    
    let info: Element<Message> = {
        let title = title_text.clone();
        let desc = desc_text.clone();
        
        Column::with_children([
            text(title).size(12).into(),
            text(desc).size(10).into(),
        ])
        .spacing(0)
        .into()
    };
    
    let bing_btn = if is_loading || selected == Source::Bing {
        button("Bing")
    } else {
        button("Bing").on_press(Message::SourceSelected(Source::Bing))
    };
    
    let spotlight_btn = if is_loading || selected == Source::Spotlight {
        button("Spotlight")
    } else {
        button("Spotlight").on_press(Message::SourceSelected(Source::Spotlight))
    };

    let refresh_btn = if is_loading {
        button("Refreshing...")
    } else {
        button("Refresh").on_press(Message::Refresh)
    };

    let settings_btn = if is_loading {
        button("Settings")
    } else {
        button("Settings").on_press(Message::SettingsOpen)
    };

    Row::with_children([
        bing_btn.into(),
        spotlight_btn.into(),
        refresh_btn.into(),
        settings_btn.into(),
        Space::new().width(Length::Fill).into(),
        info,
    ])
    .spacing(8)
    .padding(4)
    .into()
}

fn settings_view(state: &AppState) -> Element<Message> {
    container(
        Column::with_children([
            text("Settings").size(24).into(),
            Column::with_children([
                text(format!("Bing Market: {}", state.settings.bing_market)).into(),
                text(format!(
                    "Spotlight Locale: {}",
                    state.settings.spotlight_locale
                ))
                .into(),
            ])
            .spacing(12)
            .into(),
            Row::with_children([button("Close").on_press(Message::SettingsClose).into()])
                .spacing(12)
                .into(),
        ])
        .spacing(20)
        .padding(24),
    )
    .width(Length::Shrink)
    .height(Length::Shrink)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn find_latest_cached_image(cache: &Cache) -> Option<(String, String, String, PathBuf)> {
    use std::fs;
    
    let cache_dir = dirs::cache_dir()?.join("dupont");
    let mut paths: Vec<_> = fs::read_dir(cache_dir).ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "jpg" || e == "png").unwrap_or(false))
        .collect();
    
    paths.sort();
    paths.reverse();
    
    let path = paths.into_iter().next()?;
    let name = path.file_stem()?.to_string_lossy().to_string();
    Some((name, String::new(), String::new(), path))
}
