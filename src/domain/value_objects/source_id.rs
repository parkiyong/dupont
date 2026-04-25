use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceId {
    Bing,
    Spotlight,
}

impl SourceId {
    pub fn display_name(&self) -> &'static str {
        match self {
            SourceId::Bing => "Bing Wallpaper of the Day",
            SourceId::Spotlight => "Microsoft Spotlight",
        }
    }
}

impl FromStr for SourceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bing" => Ok(SourceId::Bing),
            "spotlight" => Ok(SourceId::Spotlight),
            _ => Err(format!("Unknown source: {}", s)),
        }
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceId::Bing => write!(f, "bing"),
            SourceId::Spotlight => write!(f, "spotlight"),
        }
    }
}
