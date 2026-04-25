#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size_bytes: u64,
    pub max_count: usize,
    pub max_age_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 500 * 1024 * 1024,
            max_count: 50,
            max_age_seconds: 30 * 24 * 60 * 60,
        }
    }
}
