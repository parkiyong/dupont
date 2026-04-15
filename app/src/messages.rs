#[derive(Debug)]
pub enum AppMsg {
    Refresh,
    SourceChanged(String),
}
