#[derive(Debug)]
pub enum AppMsg {
    Refresh,
    SettingsChanged {
        bing_market: String,
        spotlight_locale: String,
        show_preview: bool,
    },
}
