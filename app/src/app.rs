use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    prelude::*,
};

use crate::messages::AppMsg;

/// Command output from async background tasks.
#[derive(Debug)]
pub enum CmdOut {
    WallpaperReady {
        wallpaper: domain::Wallpaper,
        cache_path: PathBuf,
    },
    FetchError(String),
}

/// Widgets struct -- stores references to all named widgets.
pub struct Widgets {
    pub overlay: adw::ToastOverlay,
    pub preview: gtk::Picture,
    pub title_label: gtk::Label,
    pub description_label: gtk::Label,
    pub attribution_label: gtk::Label,
    pub source_dropdown: gtk::DropDown,
    pub refresh_button: gtk::Button,
}

pub struct App {
    wallpaper: Option<domain::Wallpaper>,
    cache_path: Option<PathBuf>,
    cache: Arc<tokio::sync::Mutex<domain::Cache>>,
    active_source_id: String,
    source_ids: Vec<String>,
    loading: bool,
    bing_market: Rc<RefCell<String>>,
    spotlight_locale: Rc<RefCell<String>>,
}

impl AsyncComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = CmdOut;
    type Root = adw::ApplicationWindow;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        adw::ApplicationWindow::builder()
            .title("Damask")
            .default_width(480)
            .default_height(520)
            .build()
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        // Initialize cache
        let cache = match domain::Cache::with_defaults() {
            Ok(c) => Arc::new(tokio::sync::Mutex::new(c)),
            Err(e) => {
                eprintln!("Warning: Failed to initialize cache: {}", e);
                panic!("Cannot start without a working cache: {}", e);
            }
        };

        let source_ids = vec!["bing".to_string(), "spotlight".to_string()];

        let model = App {
            wallpaper: None,
            cache_path: None,
            cache,
            active_source_id: "bing".to_string(),
            source_ids,
            loading: false,
            bing_market: Rc::new(RefCell::new("en-US".to_string())),
            spotlight_locale: Rc::new(RefCell::new("en-US".to_string())),
        };

        // Build widget tree manually
        let overlay = adw::ToastOverlay::new();

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_start(32)
            .margin_end(32)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let clamp = adw::Clamp::builder()
            .maximum_size(600)
            .build();

        let inner_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .build();

        // Preview
        let preview = gtk::Picture::builder()
            .content_fit(gtk::ContentFit::Cover)
            .hexpand(true)
            .vexpand(true)
            .build();

        // Metadata section
        let meta_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .build();

        let title_label = gtk::Label::builder()
            .css_classes(["heading"])
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .lines(1)
            .label("No wallpaper loaded")
            .build();

        let description_label = gtk::Label::builder()
            .css_classes(["body"])
            .label("Select a source and refresh to load your first wallpaper")
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .build();

        let attribution_label = gtk::Label::builder()
            .css_classes(["caption"])
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .lines(1)
            .label("")
            .build();

        // Controls section
        let controls_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();

        let source_label = gtk::Label::builder()
            .label("Source")
            .valign(gtk::Align::Center)
            .build();

        let source_names: Vec<&str> = vec!["Bing Wallpaper of the Day", "Microsoft Spotlight"];
        let string_list = gtk::StringList::new(&source_names);

        let source_dropdown = gtk::DropDown::builder()
            .model(&string_list)
            .hexpand(true)
            .enable_search(false)
            .build();

        let refresh_button = gtk::Button::builder()
            .label("Refresh Wallpaper")
            .css_classes(["suggested-action"])
            .build();

        // Assemble widget tree
        meta_box.append(&title_label);
        meta_box.append(&description_label);
        meta_box.append(&attribution_label);

        controls_box.append(&source_label);
        controls_box.append(&source_dropdown);
        controls_box.append(&refresh_button);

        inner_box.append(&preview);
        inner_box.append(&meta_box);
        inner_box.append(&controls_box);

        clamp.set_child(Some(&inner_box));
        vbox.append(&clamp);
        overlay.set_child(Some(&vbox));

        // Add header bar with settings button using adw::HeaderBar
        let header = adw::HeaderBar::new();
        header.set_show_start_title_buttons(true);
        header.set_show_end_title_buttons(true);

        let settings_btn = gtk::Button::new();
        let settings_icon = gtk::Image::from_icon_name("preferences-system-symbolic");
        settings_btn.set_child(Some(&settings_icon));
        settings_btn.set_tooltip_text(Some("Preferences"));
        header.pack_end(&settings_btn);

        // Wire settings button — recreate window each time to avoid GTK4 destroy issue
        let settings_root = root.clone();
        let settings_sender = sender.clone();
        let settings_market = model.bing_market.clone();
        let settings_locale = model.spotlight_locale.clone();
        settings_btn.connect_clicked(move |_| {
            let win = create_settings_window(
                settings_market.borrow().clone(),
                settings_locale.borrow().clone(),
                Some(&settings_root),
                settings_sender.clone(),
            );
            win.present();
        });

        // Use adw::ToolbarView to hold header bar and content
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&overlay));

        root.set_content(Some(&toolbar_view));

        // Wire source dropdown signal
        let sender_clone = sender.clone();
        let model_source_ids = model.source_ids.clone();
        source_dropdown.connect_selected_notify(move |dropdown| {
            let idx = dropdown.selected() as usize;
            if let Some(id) = model_source_ids.get(idx) {
                sender_clone.input(AppMsg::SourceChanged(id.clone()));
            }
        });

        // Wire refresh button signal
        let sender_refresh = sender.input_sender().clone();
        refresh_button.connect_clicked(move |_| {
            let _ = sender_refresh.send(AppMsg::Refresh);
        });

        // Auto-fetch initial wallpaper
        sender.input(AppMsg::Refresh);

        let widgets = Widgets {
            overlay,
            preview,
            title_label,
            description_label,
            attribution_label,
            source_dropdown,
            refresh_button,
        };

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::Refresh => {
                if self.loading {
                    return;
                }
                self.loading = true;

                // Update UI to loading state
                widgets.source_dropdown.set_sensitive(false);
                widgets.refresh_button.set_sensitive(false);
                set_button_child_spinner(&widgets.refresh_button);

                // Clone what we need for the async command
                let source_id = self.active_source_id.clone();
                let cache = self.cache.clone();
                let bing_market = self.bing_market.borrow().clone();
                let spotlight_locale = self.spotlight_locale.borrow().clone();

                // Spawn async fetch using oneshot_command
                sender.oneshot_command(async move {
                    // Create the appropriate source directly (avoids borrow-across-await)
                    let source: Box<dyn domain::Source> = if source_id == "spotlight" {
                        Box::new(domain::SpotlightSource::with_locale(spotlight_locale))
                    } else {
                        Box::new(domain::BingSource::with_market(bing_market))
                    };

                    // Fetch wallpaper
                    let wallpaper = match source.fetch().await {
                        Ok(w) => w,
                        Err(e) => {
                            return CmdOut::FetchError(e.to_string());
                        }
                    };

                    // Cache the wallpaper image (tokio::sync::Mutex is Send-safe across await)
                    let cache_path = {
                        let mut cache_guard = cache.lock().await;
                        match cache_guard.get_or_download(&wallpaper).await {
                            Ok(p) => p,
                            Err(e) => {
                                return CmdOut::FetchError(e.to_string());
                            }
                        }
                    };

                    CmdOut::WallpaperReady { wallpaper, cache_path }
                });
            }

            AppMsg::SourceChanged(id) => {
                self.active_source_id = id;
                sender.input(AppMsg::Refresh);
            }

            AppMsg::SettingsChanged { bing_market, spotlight_locale } => {
                *self.bing_market.borrow_mut() = bing_market;
                *self.spotlight_locale.borrow_mut() = spotlight_locale;
            }
        }
    }

    async fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CmdOut::WallpaperReady { wallpaper, cache_path } => {
                // Load image into preview
                let file = gtk::gio::File::for_path(&cache_path);
                if let Ok(texture) = gtk::gdk::Texture::from_file(&file) {
                    widgets.preview.set_paintable(Some(&texture));
                }

                // Update model
                self.wallpaper = Some(wallpaper.clone());
                self.cache_path = Some(cache_path.clone());

                // Update metadata labels
                widgets.title_label.set_label(&wallpaper.title);
                widgets.description_label.set_label(&wallpaper.description);
                widgets.attribution_label.set_label(&wallpaper.attribution);

                // Restore UI from loading state
                self.loading = false;
                widgets.source_dropdown.set_sensitive(true);
                widgets.refresh_button.set_sensitive(true);
                set_button_child_label(&widgets.refresh_button, "Refresh Wallpaper");
                match domain::create_desktop_backend() {
                    Ok(backend) => {
                        if let Err(e) = backend.set_wallpaper(&cache_path) {
                            eprintln!("Failed to set wallpaper: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Desktop backend not available: {}", e);
                    }
                }
            }

            CmdOut::FetchError(msg) => {
                // Show error toast
                let toast = adw::Toast::new(&msg);
                toast.set_timeout(5);
                widgets.overlay.add_toast(toast);

                // Restore UI from loading state
                self.loading = false;
                widgets.source_dropdown.set_sensitive(true);
                widgets.refresh_button.set_sensitive(true);
                set_button_child_label(&widgets.refresh_button, "Refresh Wallpaper");
            }
        }
    }
}

/// Replace the button's child widget with a spinning gtk::Spinner.
fn set_button_child_spinner(button: &gtk::Button) {
    let spinner = gtk::Spinner::builder()
        .spinning(true)
        .width_request(24)
        .height_request(24)
        .build();
    button.set_child(Some(&spinner));
}

/// Replace the button's child widget with a text label.
fn set_button_child_label(button: &gtk::Button, text: &str) {
    let label = gtk::Label::new(Some(text));
    button.set_child(Some(&label));
}

/// Create settings window with Bing market and Spotlight locale configuration.
fn create_settings_window(
    bing_market: String,
    spotlight_locale: String,
    transient_for: Option<&adw::ApplicationWindow>,
    sender: relm4::AsyncComponentSender<App>,
) -> adw::PreferencesWindow {
    let window = adw::PreferencesWindow::new();

    if let Some(parent) = transient_for {
        window.set_transient_for(Some(parent));
    }

    window.set_title(Some("Damask Preferences"));
    window.set_modal(true);

    let markets = [
        "en-US", "zh-CN", "ja-JP", "en-GB", "fr-FR", "de-DE", "pt-BR", "es-ES", "it-IT",
        "ru-RU",
    ];
    let market_names = [
        "English (US)",
        "Chinese (Simplified)",
        "Japanese",
        "English (UK)",
        "French",
        "German",
        "Portuguese (Brazil)",
        "Spanish",
        "Italian",
        "Russian",
    ];
    let market_list = gtk::StringList::new(&market_names);
    let initial_index = markets
        .iter()
        .position(|&m| m == bing_market)
        .unwrap_or(0) as u32;

    let locale_entry = adw::EntryRow::new();
    locale_entry.set_title("Locale");
    locale_entry.set_text(&spotlight_locale);

    let page = adw::PreferencesPage::new();

    let bing_group = adw::PreferencesGroup::new();
    bing_group.set_title("Bing");

    let market_row = adw::ComboRow::new();
    market_row.set_title("Market");
    market_row.set_model(Some(&market_list));
    market_row.set_selected(initial_index);

    let spotlight_group = adw::PreferencesGroup::new();
    spotlight_group.set_title("Spotlight");

    // Wire market selection → send SettingsChanged
    let sender1 = sender.clone();
    let locale_entry_clone = locale_entry.clone();
    market_row.connect_selected_notify(move |row| {
        let market = markets[row.selected() as usize].to_string();
        let locale = locale_entry_clone.text().to_string();
        sender1.input(AppMsg::SettingsChanged {
            bing_market: market,
            spotlight_locale: locale,
        });
    });

    // Wire locale entry → send SettingsChanged
    let sender2 = sender.clone();
    let market_row_clone = market_row.clone();
    locale_entry.connect_changed(move |entry| {
        let locale = entry.text().to_string();
        let market = markets[market_row_clone.selected() as usize].to_string();
        sender2.input(AppMsg::SettingsChanged {
            bing_market: market,
            spotlight_locale: locale,
        });
    });

    bing_group.add(&market_row);
    spotlight_group.add(&locale_entry);
    page.add(&bing_group);
    page.add(&spotlight_group);
    window.add(&page);

    window
}
