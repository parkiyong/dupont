use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::Mutex;

use iced::{
    Element, Length, Subscription, Task, Theme,
    time::{self, Duration},
    widget::{Column, Row, button, container, pick_list, text},
};
use iced_widget::Space;

static INIT_COUNT: AtomicU8 = AtomicU8::new(1);

use crate::application::dto::SettingsDto;
use crate::application::dto::settings_dto::{
    BING_MARKETS, SPOTLIGHT_LOCALES, bing_market_label, spotlight_locale_label,
};
use crate::application::use_cases::FetchAndSetWallpaperUseCase;
use crate::domain::{BingSource, Cache, SpotlightSource};
use crate::infrastructure::desktop::create_desktop_backend;
use crate::infrastructure::persistence::ConfigRepo;

#[derive(Clone, PartialEq, Eq)]
struct LocaleOption {
    code: String,
    label: String,
}

impl LocaleOption {
    fn new(code: &str, label_fn: fn(&str) -> String) -> Self {
        Self {
            code: code.to_string(),
            label: label_fn(code),
        }
    }
}

impl fmt::Display for LocaleOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    SourceSelected(Source),
    SettingsOpen,
    SettingsClose,
    BingMarketSelected(String),
    SpotlightLocaleSelected(String),
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
    #[allow(dead_code)]
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
    error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let cache =
            Cache::new(crate::domain::CacheConfig::default()).expect("Failed to initialize cache");
        let settings_repo = ConfigRepo::new();
        let settings = settings_repo.load();

        Self {
            cache: Arc::new(Mutex::new(cache)),
            settings_repo,
            current_wallpaper: None,
            loading: false,
            show_settings: false,
            selected_source: Source::Bing,
            settings,
            error_message: None,
        }
    }

    pub fn run() -> iced::Result {
        iced::application(AppState::new, update, view)
            .title("Dupont")
            .theme(|_: &AppState| Theme::Dark)
            .subscription(|_state| {
                if INIT_COUNT.load(Ordering::Relaxed) > 0 {
                    INIT_COUNT.fetch_sub(1, Ordering::Relaxed);
                    time::every(Duration::from_secs(1)).map(|_| Message::Refresh)
                } else {
                    Subscription::none()
                }
            })
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
                    // Create source based on selection (clone fields to avoid partial move)
                    let src: Box<dyn crate::domain::Source> = match source {
                        Source::Bing => {
                            Box::new(BingSource::with_market(settings.bing_market.clone()))
                        }
                        Source::Spotlight => Box::new(SpotlightSource::with_locale(
                            settings.spotlight_locale.clone(),
                        )),
                    };

                    // Create desktop backend
                    let desktop = match create_desktop_backend() {
                        Ok(backend) => backend,
                        Err(e) => return Err(e.to_string()),
                    };

                    // Call use case to orchestrate fetch → cache → set
                    match FetchAndSetWallpaperUseCase::execute(src, cache, desktop, &settings).await
                    {
                        Ok(output) => Ok((
                            output.title,
                            output.description,
                            output.attribution,
                            output.cache_path,
                        )),
                        Err(e) => Err(e.to_string()),
                    }
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

        Message::BingMarketSelected(market) => {
            state.settings.bing_market = market;
            let _ = state.settings_repo.save(&state.settings);
            Task::none()
        }

        Message::SpotlightLocaleSelected(locale) => {
            state.settings.spotlight_locale = locale;
            let _ = state.settings_repo.save(&state.settings);
            Task::none()
        }

        Message::WallpaperFetched(result) => {
            state.loading = false;

            match result {
                Ok((title, description, attribution, cache_path)) => {
                    state.current_wallpaper =
                        Some((title, description, attribution, cache_path.clone()));
                    state.error_message = None; // Clear any previous errors
                }
                Err(e) => {
                    log::error!("Failed to fetch wallpaper: {}", e);
                    state.error_message = Some(e);
                }
            }
            Task::none()
        }
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    if state.show_settings {
        return settings_view(state);
    }

    let controls = build_controls(state);

    // Build main content
    let content = match &state.current_wallpaper {
        Some((_, _, _, path)) => {
            let image = iced_widget::Image::new(iced_widget::image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced_core::ContentFit::Cover);

            Column::with_children([container(image).height(Length::Fill).into(), controls])
                .spacing(0)
                .padding(0)
                .into()
        }
        None => controls,
    };

    // Add error message display if present
    if let Some(error) = &state.error_message {
        Column::with_children([
            container(text(error).size(14))
                .padding(12)
                .width(Length::Fill)
                .into(),
            content,
        ])
        .spacing(0)
        .into()
    } else {
        content
    }
}

fn build_controls(state: &AppState) -> Element<'_, Message> {
    let is_loading = state.loading;
    let selected = state.selected_source;

    let (title_text, desc_text) = state
        .current_wallpaper
        .as_ref()
        .map(|(t, d, _, _)| (t.clone(), d.clone()))
        .unwrap_or((String::new(), String::new()));

    let info: Element<Message> = {
        let title = title_text.clone();
        let desc = desc_text.clone();

        Column::with_children([text(title).size(12).into(), text(desc).size(10).into()])
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

fn settings_view(state: &AppState) -> Element<'_, Message> {
    let bing_markets: Vec<LocaleOption> = BING_MARKETS
        .iter()
        .map(|s| LocaleOption::new(s, bing_market_label))
        .collect();
    let spotlight_locales: Vec<LocaleOption> = SPOTLIGHT_LOCALES
        .iter()
        .map(|s| LocaleOption::new(s, spotlight_locale_label))
        .collect();

    let bing_selected = bing_markets
        .iter()
        .find(|opt| opt.code == state.settings.bing_market)
        .cloned();
    let spotlight_selected = spotlight_locales
        .iter()
        .find(|opt| opt.code == state.settings.spotlight_locale)
        .cloned();

    let bing_picker = pick_list(bing_markets.clone(), bing_selected, |opt| {
        Message::BingMarketSelected(opt.code.clone())
    });

    let spotlight_picker = pick_list(spotlight_locales.clone(), spotlight_selected, |opt| {
        Message::SpotlightLocaleSelected(opt.code.clone())
    });

    container(
        Column::with_children([
            text("Settings").size(24).into(),
            Column::with_children([
                Row::with_children([
                    text("Bing Market:").width(Length::Fixed(130.0)).into(),
                    bing_picker.width(Length::Fixed(220.0)).into(),
                ])
                .spacing(12)
                .into(),
                Row::with_children([
                    text("Spotlight Locale:").width(Length::Fixed(130.0)).into(),
                    spotlight_picker.width(Length::Fixed(220.0)).into(),
                ])
                .spacing(12)
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
