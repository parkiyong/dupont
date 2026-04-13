# Architecture Patterns

**Domain:** Rust Desktop Wallpaper Application
**Researched:** 2025-04-13

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Main      │  │ Application │  │     UI Layer       │  │
│  │  (Entry)    │  │  (Lifecycle)│  │  (GTK/Adwaita)     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
└─────────┼──────────────────┼────────────────────┼─────────────┘
          │                  │                    │
          │                  ▼                    │
          │         ┌────────────────┐            │
          └────────►│  Core Engine   │◄───────────┘
                    │  (domain)     │
                    └────────┬───────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
      ┌──────────┐  ┌──────────┐  ┌──────────┐
      │ Sources  │  │  Cache   │  │  DE API  │
      │  (trait) │  │ Manager  │  │  Adapter │
      └──────────┘  └──────────┘  └──────────┘
          │
          ├──> Bing Source
          ├──> Spotlight Source
          └──> [future: Wallhaven, etc.]
```

### Component Boundaries

| Component | Responsibility | Communicates With | Key Trait/Interface |
|-----------|---------------|-------------------|-------------------|
| **Main** | Application entry point, setup, CLI args | Application | None |
| **Application** | App lifecycle, settings, main window | Core Engine, UI Layer | gtk::Application |
| **UI Layer** | GTK widgets, user interaction, display | Application, Core Engine | gtk::ApplicationWindow |
| **Core Engine** | Domain logic, wallpaper orchestration | Application, Sources, Cache, DE API | domain::Engine |
| **Sources (trait)** | Fetch wallpapers from APIs | Core Engine | domain::Source |
| **Cache Manager** | Download, store, retrieve images | Core Engine | domain::Cache |
| **DE API Adapter** | Set wallpaper on desktop | Core Engine | domain::DesktopEnvironment |

### Data Flow

```
1. User Action (UI)
   └─> UI emits signal/message
       └─> Application receives action
           └─> Core Engine::refresh_wallpaper(source)
               └─> Source::fetch()
                   └─> HTTP Request (async)
                       └─> Returns Wallpaper { url, metadata, ... }
                           └─> Cache::download_and_store(wallpaper)
                               └─> Local file path
                                   └─> DE API::set_wallpaper(path)
                                       └─> System call (gsettings, etc.)
                                           └─> UI updates preview

2. Manual Refresh
   UI Button ─► App::action_refresh ─► Engine::refresh ─► Source::fetch

3. Source Selection
   UI Dropdown ─► App::settings.set_string("active-source") ─► Engine::switch_source

4. Error Handling
   Any failure ─► Source returns Result<Wallpaper, Error>
               └─> UI shows toast/banner
```

## Patterns to Follow

### Pattern 1: Core/UI Separation (Plugin-Ready)

**What:** Domain logic independent of UI framework. Core engine doesn't know about GTK.

**When:** All desktop applications. Essential for swapping UI toolkits later.

**Example:**
```rust
// domain/src/source.rs - Core engine only
pub trait Source: Send + Sync {
    async fn fetch(&self) -> Result<Wallpaper, SourceError>;
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

// ui/src/widgets/source_selector.rs - UI layer only
use damask_core::Source;

pub struct SourceSelector {
    sources: Vec<Arc<dyn Source>>,
    // ... UI widgets
}
```

**Why:** Enables future swap to Iced, web UI, or TUI without touching core.

---

### Pattern 2: Async/Await with tokio

**What:** Use async for all I/O operations (HTTP, file system, DE calls).

**When:** Network requests, file downloads, blocking system calls.

**Example:**
```rust
use tokio::sync::mpsc;

pub async fn fetch_bing_wallpaper() -> Result<Wallpaper, Error> {
    let response = reqwest::get(BING_API_URL).await?;
    let data = response.json::<BingResponse>().await?;
    Ok(Wallpaper::from_bing(data))
}

// Main runtime setup
#[tokio::main]
async fn main() -> Result<()> {
    let wallpaper = fetch_bing_wallpaper().await?;
    // ...
}
```

**Why:** Non-blocking UI, good UX, Rust's async ecosystem is mature.

---

### Pattern 3: Message Passing (Channel-based)

**What:** Components communicate via channels (tokio::sync::mpsc, glib::Sender).

**When:** Cross-thread communication, async event handling.

**Example:**
```rust
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum AppMessage {
    RefreshWallpaper,
    SwitchSource(String),
    WallpaperReady(Wallpaper),
    Error(String),
}

// In Application
let (tx, mut rx) = mpsc::channel(32);

tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
        match msg {
            AppMessage::RefreshWallpaper => {
                // Handle refresh
            }
            _ => {}
        }
    }
});
```

**Why:** Decouples components, testable, handles async flows cleanly.

---

### Pattern 4: Trait-based Source Plugin System

**What:** All wallpaper sources implement a common trait. Registry manages them.

**When:** Multiple API sources, extensibility needed.

**Example:**
```rust
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn Source>>,
}

impl SourceRegistry {
    pub fn register<S: Source + 'static>(&mut self, source: S) {
        self.sources.insert(source.id().to_string(), Box::new(source));
    }

    pub fn get(&self, id: &str) -> Option<&dyn Source> {
        self.sources.get(id).map(|s| s.as_ref())
    }
}

// Usage
let mut registry = SourceRegistry::new();
registry.register(BingSource::new());
registry.register(SpotlightSource::new());
```

**Why:** Easy to add new sources, no core changes needed.

---

### Pattern 5: GTK Application Pattern (gtk-rs)

**What:** Standard GTK application lifecycle with gtk::Application and ApplicationWindow.

**When:** Any GTK-based desktop app.

**Example:**
```rust
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

fn main() {
    let app = Application::builder()
        .application_id("com.damask.Damask")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Damask")
        .default_width(400)
        .default_height(300)
        .build();

    window.present();
}
```

**Why:** Native GNOME integration, proper lifecycle management.

## Anti-Patterns to Avoid

### Anti-Pattern 1: UI in Domain Layer

**What:** Core engine depends on GTK widgets or signals.

**Why bad:** Cannot reuse core logic in CLI, TUI, or different UI toolkit. Hard to test.

**Instead:** Core engine returns data/events via Result or channels. UI observes state.

---

### Anti-Pattern 2: Blocking I/O on Main Thread

**What:** HTTP requests or file operations directly in GTK signal handlers.

**Why bad:** UI freezes, bad UX, GTK may abort.

**Instead:** All I/O in async tasks. Use `glib::spawn_future_local` or tokio runtime.

---

### Anti-Pattern 3: Global Mutable State

**What:** `static mut` or lazy_static for app state shared everywhere.

**Why bad:** Thread safety nightmares, impossible to test, violates Rust ownership.

**Instead:** Pass state through struct fields, channels, or Arc<Mutex<T>> only when needed.

---

### Anti-Pattern 4: Tight Coupling to Specific DE

**What:** Hardcoding `gsettings` calls everywhere in core engine.

**Why bad:** Cannot support COSMIC or other DEs. Core should be DE-agnostic.

**Instead:** DesktopEnvironment trait with GNOME and COSMIC implementations.

## Scalability Considerations

| Concern | 100 users (MVP) | 10K users | 1M users (unlikely for wallpaper app) |
|---------|-----------------|-----------|--------------------------------------|
| **State Management** | In-memory struct | In-memory + simple persistence | Redis/distributed cache |
| **Concurrency** | Single-threaded tokio | Multi-threaded tokio runtime | Sharded services |
| **Cache Size** | 50-100 MB | 1-2 GB | CDN-backed cache |
| **API Rate Limits** | Manual backoff | Exponential backoff + retry | Distributed throttling |
| **Configuration** | Simple YAML | YAML + GUI settings | Remote config server |

**For Damask-rs MVP:** Single-threaded tokio, in-memory state, local file cache (1-2 GB), simple YAML config.

## Desktop Environment Abstraction

### Pattern: DesktopEnvironment Trait

```rust
pub trait DesktopEnvironment: Send + Sync {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;
    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError>;
    fn name(&self) -> &'static str;
}

// GNOME implementation
pub struct GnomeBackend {
    schema: &'static str,
}

impl DesktopEnvironment for GnomeBackend {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        let mut settings = gio::Settings::new(self.schema);
        settings.set_string("picture-uri", image_path.to_str().unwrap())?;
        Ok(())
    }
}

// COSMIC implementation
pub struct CosmicBackend {
    // COSMIC uses different mechanism (to be researched)
}

impl DesktopEnvironment for CosmicBackend {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        // COSMIC-specific implementation
    }
}
```

**Why:** Easy to add new DEs, core logic unchanged. Clean separation.

## Module Structure Recommendation

```
damask-rs/
├── Cargo.toml                   # Workspace root
├── src/
│   └── main.rs                 # Application entry
├── domain/                     # Core engine (no UI)
│   ├── Cargo.toml
│   └── src/
│       ├── source.rs            # Source trait
│       ├── sources/
│       │   ├── mod.rs
│       │   ├── bing.rs
│       │   └── spotlight.rs
│       ├── cache.rs            # Download/store logic
│       ├── desktop.rs          # DE trait
│       ├── desktops/
│       │   ├── mod.rs
│       │   ├── gnome.rs
│       │   └── cosmic.rs
│       ├── engine.rs           # Orchestration
│       └── lib.rs
├── ui/                        # GTK UI layer
│   ├── Cargo.toml
│   └── src/
│       ├── application.rs      # gtk::Application
│       ├── window.rs           # gtk::ApplicationWindow
│       ├── widgets/
│       │   ├── mod.rs
│       │   ├── preview.rs
│       │   └── source_selector.rs
│       └── lib.rs
└── config/                    # Configuration management
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

## Build Order Implications

### Phase Dependencies

1. **Core Engine First** (domain crate)
   - Source trait definition
   - DesktopEnvironment trait
   - Basic cache stubs
   - Can be tested without UI

2. **Source Implementations** (domain/sources)
   - Bing API client
   - Spotlight API client
   - Integration tests for sources

3. **Desktop Backends** (domain/desktops)
   - GNOME gsettings wrapper
   - COSMIC backend (research required)
   - DE detection logic

4. **UI Layer** (ui crate)
   - GTK application structure
   - Widgets wiring to core
   - Signal handlers

5. **Integration** (application entry)
   - Wire everything together
   - Configuration loading
   - Main runtime

### Why This Order?

- **Core first** enables testing without UI complexity
- **Sources independently** testable (mockable)
- **UI last** depends on everything else
- **Parallel development possible**: Team can work on UI while others build sources

## Sources

### HIGH Confidence
- Original Damask Vala/GTK implementation (read source code directly)
- gtk-rs official documentation and examples (GitHub, 2026)
- Relm4 framework patterns (GitHub, 2026)
- Rust async ecosystem (tokio, reqwest) - well-documented

### MEDIUM Confidence
- Wallflow architecture (GitHub, 2026) - small project but well-structured
- Chameleon (Tauri-based, 2026) - different tech stack but similar problem domain

### LOW Confidence
- COSMIC desktop wallpaper API (no official docs found, will need phase-specific research)
- Best practices for large gtk-rs applications (limited examples in public repos)
