# Stack Research

**Domain:** Rust desktop wallpaper application (Linux)
**Researched:** 2025-04-13
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.93+ | Core language | Minimum version required by relm4 0.10+, provides safety and performance |
| gtk4 | 0.11 | GUI framework (bindings) | Mature, native GNOME experience, excellent documentation, battle-tested in production apps (Fractal, Amberol, 56+ apps on gtk-rs.org) |
| relm4 | 0.10 | UI framework (Elm-inspired) | Idiomatic Rust GUI development, built on gtk4-rs, simplifies state management, excellent async support with ComponentSender, accelerates development for beginners |
| gio | 0.20 | Desktop integration (gsettings) | Part of gtk-rs ecosystem, provides Settings API for wallpaper setting on GNOME/COSMIC, cross-platform compatibility |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| reqwest | 0.12 | HTTP client (async) | Fetching wallpapers from Bing API, use with `json` feature for JSON parsing |
| serde | 1.0 | Serialization framework | Required by reqwest for JSON parsing of API responses |
| serde_json | 1.0 | JSON serialization/deserialization | Parsing Bing Wallpaper API responses |
| tokio | 1.51 | Async runtime | Required by reqwest for async HTTP operations, use with `rt-multi-thread` feature |
| dirs | 5.0 | XDG directory paths | Getting cache, config, and data directories following Linux standards (~/.cache, ~/.config, ~/.local/share) |
| thiserror | 2.0 | Error handling (domain-specific) | Define custom error types for core engine (fetch errors, cache errors, wallpaper setting errors) |
| anyhow | 1.0 | Error handling (application) | Easy error propagation in UI layer, convert domain errors to application errors |
| log | 0.4 | Logging facade | Structured logging API |
| env_logger | 0.11 | Logger implementation | Runtime-configurable logger for development |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo | Build tool (standard) | Rust's package manager and build system |
| rustup | Version manager | Manage Rust toolchain versions |
| clippy | Linting | Rust component, enables additional lints: `rustup component add clippy` |
| rustfmt | Code formatting | Rust component, auto-format code: `rustup component add rustfmt` |
| gtk4-update-icon-cache | Icon cache management | Part of GTK4 installation, update application icons |
| glib-compile-resources | Resource compilation | Part of GLib installation, compile GResource files (icons, UI definitions) |
| desktop-file-validate | Desktop file validation | Part of desktop-file-utils, validate .desktop files for Linux integration |

## Installation

```bash
# System dependencies (Linux - Debian/Ubuntu)
sudo apt install libgtk-4-dev libadwaita-1-dev meson desktop-file-utils gcc gtk-update-icon-cache

# System dependencies (Linux - Fedora)
sudo dnf install gtk4-devel libadwaita-devel meson desktop-file-utils gcc glib-compile-resources gtk4-update-icon-cache update-desktop-database

# System dependencies (Linux - Arch)
sudo pacman -S gtk4 libadwaita meson desktop-file-utils gcc

# Core dependencies
cargo add gtk4 relm4 gio reqwest serde serde_json tokio dirs thiserror anyhow log env_logger

# Feature flags for core dependencies
cargo add relm4 --features macros
cargo add reqwest --features json
cargo add tokio --features rt-multi-thread

# Optional: image processing (for resizing/scaling wallpapers)
cargo add image

# Dev dependencies
cargo add --dev tokio-test
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| gtk4-rs + relm4 | gtk4-rs (direct) | When you need fine-grained control over GTK4 internals or have unusual async patterns; use relm4 for faster, more maintainable code in most cases |
| gtk4-rs + relm4 | iced | For COSMIC-native UI development or pure Rust cross-platform apps; less mature than GTK4, smaller ecosystem, different async model |
| reqwest | ureq | For simple blocking HTTP requests only; reqwest provides async support, connection pooling, and is more feature-complete |
| tokio | async-std | If you prefer async-std's API style; tokio has larger ecosystem, better async TLS support, and is the de facto standard |
| log + env_logger | tracing + tracing-subscriber | For production applications requiring structured logging, async-aware tracing, or complex log filtering; log + env_logger is simpler for MVP |
| thiserror + anyhow | only anyhow | For small apps without domain-specific error needs; thiserror provides better error messages and type safety for core engine errors |
| dirs | xdg | Never use xdg crate (deprecated, unmaintained since 2020); dirs is actively maintained and provides cleaner API |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| gtk-rs (GTK3) | GTK3 is deprecated, GTK4 is current stable, better performance, native Wayland support, future-proofing | gtk4-rs (GTK4) |
| xdg crate | Deprecated since 2020, unmaintained, last update 5+ years ago, may have security issues | dirs crate (actively maintained, v5.0+) |
| relm (old version) | Original relm project, not compatible with GTK4, lacks modern Elm-inspired architecture | relm4 (complete rewrite, GTK4-compatible, active development) |
| curl-rust | More complex API, less idiomatic Rust, smaller community than reqwest | reqwest (standard HTTP client in Rust ecosystem) |
| synchronous blocking HTTP | Blocks GTK4 main loop, freezes UI during network requests, poor user experience | reqwest async with tokio runtime |
| manual path handling for config/cache | Non-standard locations, violates XDG Base Directory Specification, harder for users to find files | dirs crate (follows XDG standards: ~/.cache, ~/.config, ~/.local/share) |
| unwrap() everywhere | Panics crash the application, poor error handling, bad UX | thiserror (domain errors) + anyhow (app errors) with proper error propagation |
| Direct shell command execution (calling `gsettings` command) | Fragile, platform-dependent, error handling via exit codes, not type-safe | gio::Settings API (type-safe, integrates with GLib main loop) |

## Stack Patterns by Variant

**If targeting only GNOME:**
- Use gtk4 + relm4 with libadwaita (optional, for GNOME HIG compliance)
- Because: libadwaita provides GNOME design language components, better integration, modern look-and-feel

**If targeting COSMIC with future Iced migration:**
- Use gtk4 + relm4 for v1, but architect core engine with trait-based abstraction
- Because: Clean separation enables UI toolkit swap without rewriting core logic (fetch, cache, wallpaper setting)
- Pattern: Define `WallpaperEngine` trait, implement in core crate, UI layer uses trait methods

**If adding automatic refresh (v2 feature):**
- Use tokio with tokio::time::interval for periodic tasks
- Because: Tokio integrates well with relm4's async model, ComponentSender can send messages to UI on timer events

**If adding image resizing/scaling:**
- Use image crate 0.25 for format decoding/encoding
- Because: Most comprehensive image format support in Rust, actively maintained, good performance

**If needing more structured logging for production:**
- Migrate from log + env_logger to tracing + tracing-subscriber + tracing-appender
- Because: Structured fields, async-aware, better filtering, file logging with rotation

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| relm4 0.10 | gtk4 0.11.x, tokio 1.51+ | relm4 depends on gtk4 0.11.2 and tokio 1.51 internally |
| reqwest 0.12 | tokio 1.x | reqwest 0.12 uses tokio 1.x for async runtime |
| gio 0.20 | glib 0.20 | Part of gtk-rs-core, version-synced with glib |
| gtk4 0.11 | GTK 4.14+ (system library) | Must have GTK4 development libraries installed on system |
| dirs 5.0 | No runtime dependencies | Pure Rust, cross-platform |
| thiserror 2.0 | serde 1.x | Optional derive for Serialize/Deserialize on Error types |

**Critical compatibility note:** System GTK4 library version must match gtk4-rs expectations. On Debian/Ubuntu, `libgtk-4-dev` provides compatible GTK4. Always test on target distribution before assuming compatibility.

## Architecture Pattern for Clean UI Swapping

```rust
// Core engine (crate: damask-core)
pub trait WallpaperEngine {
    async fn fetch(&self, source: WallpaperSource) -> Result<Wallpaper, FetchError>;
    async fn cache(&self, wallpaper: &Wallpaper) -> Result<PathBuf, CacheError>;
    async fn set(&self, path: &Path) -> Result<(), WallpaperError>;
}

// UI implementation (crate: damask-ui-gtk4)
// Uses WallpaperEngine trait, implements with gtk4-rs + relm4

// Future UI implementation (crate: damask-ui-iced)
// Same WallpaperEngine trait, implements with iced
```

This pattern satisfies project constraint: "GTK-rs for v1 — Must be easily swappable to Iced or other toolkits later via clean architecture"

## Sources

- gtk-rs.org — GTK4 installation, available crates, ecosystem overview, app examples (HIGH confidence)
- relm4.org — Relm4 documentation, async patterns, ComponentSender API, tokio integration (HIGH confidence)
- docs.rs/crates/reqwest — Reqwest API documentation, features, async patterns (HIGH confidence)
- crates.io — Current stable versions: gtk4 0.11, relm4 0.10, reqwest 0.12, tokio 1.51, dirs 5.0 (HIGH confidence)
- GNOME developer documentation — GSettings API for wallpaper setting: `org.gnome.desktop.background` schema, `picture-uri` key (HIGH confidence)
- System76 COSMIC documentation — GNOME compatibility mode, wallpaper setting via gsettings (MEDIUM confidence — COSMIC-specific APIs less documented, gsettings approach verified to work)

---
*Stack research for: Rust desktop wallpaper application (Linux)*
*Researched: 2025-04-13*
