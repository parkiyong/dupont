use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct SettingsModel {
    pub bing_market: String,
    pub spotlight_locale: String,
}

pub struct SettingsWidgets {
    pub market_row: adw::ComboRow,
    pub locale_entry: adw::EntryRow,
}

#[derive(Debug)]
pub enum SettingsInput {
    Show,
}

#[derive(Debug)]
pub enum SettingsOutput {
    SettingsChanged {
        bing_market: String,
        spotlight_locale: String,
    },
}

pub struct SettingsComponent;

impl SimpleComponent for SettingsComponent {
    type Init = (String, String);
    type Input = SettingsInput;
    type Output = SettingsOutput;
    type Root = adw::PreferencesWindow;
    type Widgets = SettingsWidgets;

    fn init_root() -> Self::Root {
        adw::PreferencesWindow::new()
    }

    fn init(
        (bing_market, spotlight_locale): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsModel {
            bing_market,
            spotlight_locale,
        };

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
            .position(|&m| m == model.bing_market)
            .unwrap_or(0) as u32;

        let locale_entry = adw::EntryRow::new();
        locale_entry.set_title("Locale");
        locale_entry.set_text(&model.spotlight_locale);

        let page = adw::PreferencesPage::new();

        let bing_group = adw::PreferencesGroup::new();
        bing_group.set_title("Bing");

        let market_row = adw::ComboRow::new();
        market_row.set_title("Market");
        market_row.set_model(Some(&market_list));
        market_row.set_selected(initial_index);

        let spotlight_group = adw::PreferencesGroup::new();
        spotlight_group.set_title("Spotlight");

        bing_group.add(&market_row);
        spotlight_group.add(&locale_entry);
        page.add(&bing_group);
        page.add(&spotlight_group);
        root.add(&page);
        root.set_title(Some("Dupont Preferences"));
        root.set_modal(true);

        let sender_apply = sender.output_sender().clone();
        let markets_vec = markets.to_vec();
        market_row.connect_selected_notify(move |combo| {
            let sender = sender_apply.clone();
            let locale = locale_entry.text().to_string();
            let markets = markets_vec.clone();
            let idx = combo.selected() as usize;
            let market = markets
                .get(idx)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "en-US".to_string());
            let _ = sender.send(SettingsOutput::SettingsChanged {
                bing_market: market,
                spotlight_locale: locale,
            });
        });

        let sender_locale = sender.output_sender().clone();
        let markets_vec2 = markets.to_vec();
        locale_entry.connect_changed(move |entry| {
            let sender = sender_locale.clone();
            let locale = entry.text().to_string();
            let markets = markets_vec2.clone();
            let market_idx = market_row.selected() as usize;
            let market = markets
                .get(market_idx)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "en-US".to_string());
            let _ = sender.send(SettingsOutput::SettingsChanged {
                bing_market: market,
                spotlight_locale: locale,
            });
        });

        let widgets = SettingsWidgets {
            market_row,
            locale_entry,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SettingsInput::Show => {}
        }
    }
}
