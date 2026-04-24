use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    prelude::*,
};

use crate::app::messages::AppMsg;

/// Command output from async background tasks.
#[derive(Debug)]
pub enum CmdOut {
    WallpaperReady {
        wallpaper: crate::domain::Wallpaper,
        cache_path: PathBuf,
    },
    FetchError(String),
}

/// Widgets struct -- stores references to all named widgets.
pub struct Widgets {
    pub preview: gtk::Picture,
    pub title_label: gtk::Label,
    pub description_label: gtk::Label,
    pub attribution_label: gtk::Label,
    pub source_dropdown: gtk::DropDown,
    pub refresh_button: gtk::Button,
}

pub struct WidgetsPlus {
    pub widgets: Widgets,
    pub source_ids: Vec<String>,
}

pub struct App {
    wallpaper: Option<crate::domain::Wallpaper>,
    cache_path: Option<PathBuf>,
    cache: Arc<tokio::sync::Mutex<crate::domain::Cache>>,
    loading: bool,
    bing_market: Rc<RefCell<String>>,
    spotlight_locale: Rc<RefCell<String>>,
    show_preview: Rc<RefCell<bool>>,
}

impl AsyncComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = CmdOut;
    type Root = gtk::ApplicationWindow;
    type Widgets = WidgetsPlus;

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title("Dupont")
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
        let cache = match crate::domain::Cache::with_defaults() {
            Ok(c) => Arc::new(tokio::sync::Mutex::new(c)),
            Err(e) => {
                eprintln!("Warning: Failed to initialize cache: {}", e);
                panic!("Cannot start without a working cache: {}", e);
            }
        };

        // Load persisted config (falls back to defaults)
        let cfg = crate::app::config::Config::load();

        let model = App {
            wallpaper: None,
            cache_path: None,
            cache,
            loading: false,
            bing_market: Rc::new(RefCell::new(cfg.bing_market)),
            spotlight_locale: Rc::new(RefCell::new(cfg.spotlight_locale)),
            show_preview: Rc::new(RefCell::new(cfg.show_preview)),
        };

        // Build widget tree manually
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(18)
            .margin_start(32)
            .margin_end(32)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let clamp = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
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

        clamp.append(&inner_box);
        vbox.append(&clamp);

        // Add header bar with settings button
        let header = gtk::HeaderBar::new();
        header.set_show_title_buttons(true);

        let settings_btn = gtk::Button::new();
        let settings_icon = gtk::Image::from_icon_name("preferences-system-symbolic");
        settings_btn.set_child(Some(&settings_icon));
        settings_btn.set_tooltip_text(Some("Preferences"));
        header.pack_end(&settings_btn);

        // Wire settings button
        let settings_root = root.clone();
        let settings_sender = sender.clone();
        let settings_market = model.bing_market.clone();
        let settings_locale = model.spotlight_locale.clone();
        let settings_preview = model.show_preview.clone();
        settings_btn.connect_clicked(move |_| {
            let win = create_settings_window(
                settings_market.borrow().clone(),
                settings_locale.borrow().clone(),
                *settings_preview.borrow(),
                Some(&settings_root),
                settings_sender.clone(),
            );
            win.present();
        });

        root.set_titlebar(Some(&header));
        root.set_child(Some(&vbox));

        // Wire refresh button signal
        let sender_refresh = sender.input_sender().clone();
        refresh_button.connect_clicked(move |_| {
            let _ = sender_refresh.send(AppMsg::Refresh);
        });

        // Set dropdown to default (bing, index 0)
        source_dropdown.set_selected(0);

        let source_ids = vec!["bing".to_string(), "spotlight".to_string()];

        let widgets = Widgets {
            preview,
            title_label,
            description_label,
            attribution_label,
            source_dropdown,
            refresh_button,
        };

        let widgets_plus = WidgetsPlus {
            widgets,
            source_ids,
        };

        AsyncComponentParts { model, widgets: widgets_plus }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut WidgetsPlus,
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
                widgets.widgets.source_dropdown.set_sensitive(false);
                widgets.widgets.refresh_button.set_sensitive(false);
                set_button_child_spinner(&widgets.widgets.refresh_button);

                // Get source from dropdown selection
                let idx = widgets.widgets.source_dropdown.selected() as usize;
                let source_id = widgets.source_ids.get(idx).cloned().unwrap_or_else(|| "bing".to_string());

                // Clone what we need for the async command
                let cache = self.cache.clone();
                let bing_market = self.bing_market.borrow().clone();
                let spotlight_locale = self.spotlight_locale.borrow().clone();

                // Spawn async fetch using oneshot_command
                sender.oneshot_command(async move {
                    // Create the appropriate source directly (avoids borrow-across-await)
                    let source: Box<dyn crate::domain::Source> = if source_id == "spotlight" {
                        Box::new(crate::domain::SpotlightSource::with_locale(spotlight_locale))
                    } else {
                        Box::new(crate::domain::BingSource::with_market(bing_market))
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

            AppMsg::SettingsChanged { bing_market, spotlight_locale, show_preview } => {
                *self.bing_market.borrow_mut() = bing_market.clone();
                *self.spotlight_locale.borrow_mut() = spotlight_locale.clone();
                *self.show_preview.borrow_mut() = show_preview;
                let cfg = crate::app::config::Config {
                    bing_market,
                    spotlight_locale,
                    show_preview,
                };
                if let Err(e) = cfg.save() {
                    eprintln!("Failed to save config: {}", e);
                }
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
widgets.widgets.preview.set_paintable(Some(&texture));
                }

                widgets.widgets.title_label.set_label(&wallpaper.title);
                widgets.widgets.description_label.set_label(&wallpaper.description);
                widgets.widgets.attribution_label.set_label(&wallpaper.attribution);

                // Restore UI from loading state
                self.loading = false;
                widgets.widgets.source_dropdown.set_sensitive(true);
                widgets.widgets.refresh_button.set_sensitive(true);
                set_button_child_label(&widgets.widgets.refresh_button, "Refresh Wallpaper");
                match crate::domain::create_desktop_backend() {
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
                eprintln!("Error: {}", msg);

                // Restore UI from loading state
                self.loading = false;
                widgets.widgets.source_dropdown.set_sensitive(true);
                widgets.widgets.refresh_button.set_sensitive(true);
                set_button_child_label(&widgets.widgets.refresh_button, "Refresh Wallpaper");
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
    show_preview: bool,
    transient_for: Option<&gtk::ApplicationWindow>,
    sender: relm4::AsyncComponentSender<App>,
) -> gtk::Window {
    let window = gtk::Window::new();

    if let Some(parent) = transient_for {
        window.set_transient_for(Some(parent));
    }

    window.set_title(Some("Dupont Preferences"));
    window.set_modal(true);
    window.set_default_size(400, 350);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let markets = [
        "en-US", "zh-CN", "ja-JP", "en-AU", "en-GB", "de-DE", "en-NZ", "en-CA", "en-IN",
        "fr-FR", "fr-CA",
    ];
    let market_names = [
        "English (United States)",
        "Chinese (Simplified)",
        "Japanese (Japan)",
        "English (Australia)",
        "English (United Kingdom)",
        "German (Germany)",
        "English (New Zealand)",
        "English (Canada)",
        "English (India)",
        "French (France)",
        "French (Canada)",
    ];
    let market_list = gtk::StringList::new(&market_names);
    
    // Bing Market
    let bing_label = gtk::Label::new(Some("Bing Market"));
    bing_label.set_halign(gtk::Align::Start);
    vbox.append(&bing_label);

    let market_dropdown = gtk::DropDown::builder()
        .model(&market_list)
        .build();
    let initial_index = markets
        .iter()
        .position(|&m| m == bing_market)
        .unwrap_or(0) as u32;
    market_dropdown.set_selected(initial_index);
    vbox.append(&market_dropdown);

    // Spotlight Locale
    let spotlight_label = gtk::Label::new(Some("Spotlight Locale"));
    spotlight_label.set_halign(gtk::Align::Start);
    vbox.append(&spotlight_label);

    let locale_list = gtk::StringList::new(&market_names);
    let locale_dropdown = gtk::DropDown::builder()
        .model(&locale_list)
        .build();
    let locale_initial_index = markets
        .iter()
        .position(|&m| m == spotlight_locale)
        .unwrap_or(0) as u32;
    locale_dropdown.set_selected(locale_initial_index);
    vbox.append(&locale_dropdown);

    // Show Preview Toggle
    let preview_check = gtk::CheckButton::with_label("Show preview before changing wallpaper");
    preview_check.set_active(show_preview);
    vbox.append(&preview_check);

    // Wire signals
    let sender1 = sender.clone();
    let locale_dropdown_clone = locale_dropdown.clone();
    let preview_check_clone1 = preview_check.clone();
    market_dropdown.connect_selected_notify(move |dropdown| {
        let market = markets[dropdown.selected() as usize].to_string();
        let locale = markets[locale_dropdown_clone.selected() as usize].to_string();
        sender1.input(AppMsg::SettingsChanged {
            bing_market: market,
            spotlight_locale: locale,
            show_preview: preview_check_clone1.is_active(),
        });
    });

    let sender2 = sender.clone();
    let market_dropdown_clone = market_dropdown.clone();
    let preview_check_clone2 = preview_check.clone();
    locale_dropdown.connect_selected_notify(move |dropdown| {
        let locale = markets[dropdown.selected() as usize].to_string();
        let market = markets[market_dropdown_clone.selected() as usize].to_string();
        sender2.input(AppMsg::SettingsChanged {
            bing_market: market,
            spotlight_locale: locale,
            show_preview: preview_check_clone2.is_active(),
        });
    });

    let sender3 = sender.clone();
    let market_dropdown_clone3 = market_dropdown.clone();
    let locale_dropdown_clone3 = locale_dropdown.clone();
    preview_check.connect_toggled(move |check| {
        let market = markets[market_dropdown_clone3.selected() as usize].to_string();
        let locale = markets[locale_dropdown_clone3.selected() as usize].to_string();
        sender3.input(AppMsg::SettingsChanged {
            bing_market: market,
            spotlight_locale: locale,
            show_preview: check.is_active(),
        });
    });

    window.set_child(Some(&vbox));
    window
}
