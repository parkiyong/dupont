# Phase 3: UI Layer Implementation - Research

**Researched:** 2026-04-14
**Domain:** relm4 + gtk4-rs + libadwaita desktop UI
**Confidence:** HIGH

## Summary

Phase 3 builds the GTK4 presentation layer using relm4 0.11.0 as the component framework. The phase creates a new `app` crate that wraps the existing `domain` crate with a native Linux UI. All six requirements (UI-01 through UI-06) are implementable with the standard relm4 + libadwaita stack: `adw::ApplicationWindow` for the main shell, `gtk::Picture` for wallpaper preview, `adw::ComboRow` for source selection, `gtk::Button` for refresh, `adw::ToastOverlay` for error messages, and `adw::PreferencesWindow` for settings.

The domain crate is fully UI-agnostic -- all types are `Send + Sync`, errors implement `Display` via thiserror, and async operations use tokio. relm4's `AsyncComponent` trait bridges cleanly to this: its `update()` method receives an `AsyncComponentSender` that can spawn tokio tasks, and the sender sends messages back to `update()` when tasks complete. The `init_loading_widgets()` method provides a built-in spinner during async initialization.

One notable gap: `Wallpaper::thumbnail_url` is `Option<String>` but neither `BingSource` nor `SpotlightSource` currently sets it. The UI must use `wallpaper.url` (full image URL) for preview, or the planner should include a task to populate `thumbnail_url` during fetch.

**Primary recommendation:** Use `AsyncComponent` (not `Component` + `Commands`) for the main window. The async `init()` method naturally handles startup tasks (registry setup, cache init, initial fetch), and `init_loading_widgets()` gives a free loading spinner.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use relm4 as the component framework on top of gtk4-rs -- Component trait with init/update pattern, message passing, built-in async support via Commands
- **D-02:** Preview-focused layout -- large wallpaper preview fills most of the window (main content), metadata displayed below the preview, source selector and refresh button at the bottom as controls
- **D-03:** Use relm4's Command system for async operations -- update() dispatches Commands that send messages back when done, loading state tracked via component model field
- **D-04:** Use adw::ToastOverlay wrapping main content -- errors shown as temporary toast messages at the bottom that auto-dismiss, non-blocking, matches GNOME HIG

### Claude's Discretion
- Exact widget sizing and spacing
- Loading state visual (spinner vs skeleton)
- Toast duration and detail level
- Settings window layout and widget organization
- Metadata display formatting

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-01 | Wallpaper preview (300x200px thumbnail) | `gtk::Picture` with `set_content_fit(ContentFit::Cover)`. Use `wallpaper.url` (full image) since `thumbnail_url` is unpopulated. Load via `gio::File` + `gtk::Texture` from cached file path. |
| UI-02 | Metadata display (title, description, attribution) | `gtk::Label` widgets stacked below preview. `Wallpaper` struct has `title`, `description`, `attribution` fields ready for display. |
| UI-03 | Manual refresh button | `gtk::Button` with label/icon. On click, dispatch async task via `sender.oneshot_command()` calling `source.fetch()` then `cache.get_or_download()`. |
| UI-04 | Source selector dropdown | `adw::ComboRow` in `adw::ActionRow` populated from `SourceRegistry::list()`. On change, switch active source and trigger fetch. |
| UI-05 | Error toasts for all failures | `adw::ToastOverlay` wrapping main content. `Toast::new(err.to_string())` with 3-second timeout. Domain errors implement `Display` via thiserror. |
| UI-06 | Settings window | `adw::PreferencesWindow` with `adw::EntryRow` for Bing market (default "en-US") and Spotlight locale (default "80217"). Apply recreates source instances. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| relm4 | 0.11.0 | Component framework (Elm architecture, async support) | De facto standard Rust GTK4 framework; Active maintainer (Aaron Erhardt); Published 2026-04-08 |
| gtk4 | 0.11.2 | GTK4 Rust bindings | Required by relm4; system-installed GTK 4.20.3 |
| libadwaita | 0.9.1 | GNOME HIG widgets (ToastOverlay, PreferencesWindow) | Per D-04 decision; system-installed 1.8.4 |
| relm4-macros | 0.11.0 | Procedural macros (view!, component) | Required by relm4 for view macro expansion |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| domain (workspace) | local | Business logic (Source, Cache, DesktopEnvironment) | Every UI operation calls domain; never duplicated |
| gio | 0.20 (workspace) | File loading for gtk::Picture | Loading cached wallpaper images from disk |
| tokio | 1.51 (workspace) | Async runtime | relm4 manages its own runtime; domain calls use tokio internally |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| AsyncComponent | Component + Commands | D-03 mentions Commands, but AsyncComponent is strictly better here: async init(), init_loading_widgets(), cleaner ergonomics. Commands are the older pattern. |
| gtk::DropDown | adw::ComboRow | ComboRow is a libadwaita-styled dropdown with built-in label, matching GNOME HIG. DropDown is lower-level. |
| gtk::Window | adw::ApplicationWindow | ApplicationWindow provides libadwaita theming, desktop integration, and works with ToastOverlay correctly. |

**Installation:**
```toml
# In app/Cargo.toml
[dependencies]
relm4 = { version = "0.11", features = ["libadwaita", "gnome_49"] }
gtk = { version = "0.11", package = "gtk4" }
adw = { version = "0.9", package = "libadwaita" }
domain = { path = "../domain" }
```

**Feature flag rationale:** `gnome_49` enables `adw/v1_8` which matches the system-installed libadwaita 1.8.4. Using `gnome_50` would require libadwaita 1.9+ which is not installed. [VERIFIED: crates.io API + system pkg-config]

**Version verification:**
```
relm4:     0.11.0 (published 2026-04-08) [VERIFIED: crates.io API]
gtk4-rs:   0.11.2 (published 2026-04-03) [VERIFIED: crates.io API]
libadwaita: 0.9.1 (published 2026-04-04) [VERIFIED: crates.io API]
relm4-components: 0.11.0 (published 2026-04-08) [VERIFIED: crates.io API]
```

## Architecture Patterns

### Recommended Project Structure
```
app/
  Cargo.toml
  src/
    main.rs              # Application entry point, RelmApp::new().run()
    app.rs               # Top-level AsyncComponent (AppComponent)
    widgets/
      mod.rs
      preview.rs         # Wallpaper preview widget (gtk::Picture + metadata labels)
      controls.rs        # Source selector + refresh button row
      settings.rs        # PreferencesWindow component
    model.rs             # App model (wallpaper state, source registry, cache, loading)
    messages.rs          # Message enum (Refresh, SetSource, WallpaperLoaded, Error, etc.)
```

### Pattern 1: AsyncComponent for Main Window

**What:** The main application window uses `AsyncComponent` which provides async `init()` and `update()` methods. This is the recommended pattern for components that need async work during initialization or in response to events.

**When to use:** Any component that needs to call domain async methods (fetch, cache, desktop operations).

**Key insight:** D-03 mentions "Command system" but `AsyncComponent` is the modern relm4 approach that supersedes the `Component` + `Command` pattern. Both work, but `AsyncComponent` provides:
- Async `init()` for startup tasks (registry setup, cache init)
- `init_loading_widgets()` for a free loading spinner
- `sender.oneshot_command()` for spawning async tasks from `update()`

**Example:**
```rust
// Source: relm4 book - AsyncComponent pattern
use relm4::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};

struct AppModel {
    wallpaper: Option<Wallpaper>,
    cache: Option<Cache>,
    registry: SourceRegistry,
    active_source: String,
    loading: bool,
}

#[derive(Debug)]
enum AppMsg {
    Refresh,
    SourceChanged(String),
    WallpaperLoaded(Result<Wallpaper, SourceError>),
    ApplyWallpaper(Result<(), DEError>),
    Error(String),
}

#[relm4::component(async)]
impl AsyncComponent for AppComponent {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Damask"),
            set_default_size: (500, 600),

            #[name = "overlay"]
            adw::ToastOverlay {
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 16,

                    // Preview area
                    #[name = "preview"]
                    gtk::Picture {
                        set_content_fit: gtk::ContentFit::Cover,
                        set_hexpand: true,
                        set_vexpand: true,
                    },

                    // Metadata
                    #[name = "title_label"]
                    gtk::Label {
                        set_label: "No wallpaper loaded",
                        add_css_class: "title-2",
                    },

                    // Controls
                    adw::ActionRow {
                        set_title: "Source",
                        #[wrap(Some)]
                        set_suffix = &adw::ComboRow {
                            set_title: "Wallpaper source",
                            #[watch]
                            set_model: &source_list,
                            connect_selected_notify[sender] => move |combo| {
                                // Get selected item and send SourceChanged message
                            },
                        }
                    },

                    gtk::Button {
                        set_label: "Refresh",
                        set_halign: gtk::Align::Center,
                        connect_clicked => AppMsg::Refresh,
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AppModel {
            wallpaper: None,
            cache: Cache::with_defaults().ok(),
            registry: SourceRegistry::new(),
            active_source: "bing".to_string(),
            loading: true,
        };

        let widgets = view!();
        widgets.overlay.add_toast(&adw::Toast::new("Loading..."));

        // Kick off initial fetch
        sender.input(AppMsg::Refresh);
        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::Refresh => {
                self.loading = true;
                let source = self.registry.get(&self.active_source);
                if let Some(source) = source {
                    let result = source.fetch().await;
                    match result {
                        Ok(wallpaper) => {
                            // Download and cache
                            if let Some(cache) = self.cache.as_mut() {
                                match cache.get_or_download(&wallpaper).await {
                                    Ok(path) => {
                                        self.wallpaper = Some(wallpaper);
                                        // Load image from cached path
                                    }
                                    Err(e) => {
                                        // Show error toast
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Show error toast with e.to_string()
                        }
                    }
                }
                self.loading = false;
            }
            AppMsg::SourceChanged(source_id) => {
                self.active_source = source_id;
                // Trigger refresh with new source
            }
            // ...
        }
    }
}
```

### Pattern 2: ToastOverlay for Error Handling

**What:** `adw::ToastOverlay` wraps the main content area. Errors are displayed as `adw::Toast` instances that auto-dismiss after a configurable timeout. This is non-blocking and matches GNOME HIG.

**When to use:** All error display (D-04 decision).

**Example:**
```rust
// Source: libadwaita 1.x API
// In view! macro, give ToastOverlay a name
#[name = "overlay"]
adw::ToastOverlay { ... }

// In update(), to show an error:
fn show_error(&self, overlay: &adw::ToastOverlay, message: impl Into<glib::GString>) {
    let toast = adw::Toast::new(&message.into());
    toast.set_timeout(3); // 3 seconds auto-dismiss
    overlay.add_toast(&toast);
}
```

### Pattern 3: Loading State with init_loading_widgets

**What:** `AsyncComponent` provides `init_loading_widgets()` which displays temporary widgets (typically a spinner) while async `init()` is running. This handles the startup loading state for free.

**Example:**
```rust
fn init_loading_widgets(root: &Self::Root) -> Option<gtk::Widget> {
    let spinner = gtk::Spinner::new();
    spinner.set_spinning(true);
    spinner.set_halign(gtk::Align::Center);
    spinner.set_valign(gtk::Align::Center);
    Some(spinner.into())
}
```

### Anti-Patterns to Avoid
- **Blocking update() with long async operations:** Always spawn async work via `sender.oneshot_command()` or keep the update loop non-blocking. Never `.await` a long network call directly in `update()` without the sender pattern.
- **Cloning large structs into closures:** Clone only what's needed (source ID strings, not the entire model). Use `Arc` if shared ownership is required.
- **Manual widget state management:** Use `#[watch]` attribute in the `view!` macro to automatically track model changes and update widget properties. Don't manually call `set_*` in `update()`.
- **Using Component instead of AsyncComponent for async work:** The `Component` + `Command` pattern is the older approach. `AsyncComponent` is cleaner for this use case since every user action triggers async domain calls.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Elm architecture (model/update/view) | Custom message-passing system | relm4 `AsyncComponent` trait + `view!` macro | Handles sender/receiver, widget lifecycle, CSS, and async bridging |
| Loading spinner | Manual gtk::Spinner management with timeout | `AsyncComponent::init_loading_widgets()` | Built-in pattern with automatic removal when init completes |
| Error toast system | Custom popup window or dialog | `adw::ToastOverlay` + `adw::Toast` | Matches GNOME HIG, auto-dismiss, non-blocking, accessible |
| Settings window layout | Manual grid/box of form fields | `adw::PreferencesWindow` + `adw::EntryRow` | GNOME-standard settings UI with group headers, search, and responsive layout |
| Dropdown selector | Manual gtk::ListBox with selection | `adw::ComboRow` | libadwaita-styled combo box with label, integrates into preferences groups |
| Image scaling for preview | Manual aspect-ratio calculation | `gtk::Picture` with `set_content_fit(ContentFit::Cover)` | Handles scaling, aspect ratio, and memory management |
| Async-to-GUI bridging | Manual channel/glib::idle_add | relm4 `AsyncComponentSender` | Sender queues messages to the GTK main thread automatically |

**Key insight:** relm4 + libadwaita covers every UI pattern needed for this phase. There is zero reason to hand-roll any presentation logic.

## Common Pitfalls

### Pitfall 1: thumbnail_url is None for all sources
**What goes wrong:** Code tries to load `wallpaper.thumbnail_url` for preview and gets `None`, showing a blank preview area.
**Why it happens:** Neither `BingSource::fetch()` nor `SpotlightSource::fetch()` calls `wallpaper.with_thumbnail()`. The field is always `None`.
**How to avoid:** Use `wallpaper.url` (full image URL) for preview. The `Cache::get_or_download()` method downloads the full image and returns a local `PathBuf` -- load from that path.
**Warning signs:** Preview always blank; `Option<String>` unwrap panic.

### Pitfall 2: Blocking the GTK main thread with async domain calls
**What goes wrong:** UI freezes during wallpaper fetch (network latency 1-10 seconds).
**Why it happens:** Calling `.await` directly in `update()` or a signal handler blocks the GTK event loop if not dispatched correctly.
**How to avoid:** Use `sender.oneshot_command()` to spawn async tasks. The task runs on a background thread and sends a message back to `update()` when complete. relm4 manages the runtime.
**Warning signs:** UI becomes unresponsive during refresh; system may show "Application Not Responding".

### Pitfall 3: Feature flag mismatch with system libadwaita
**What goes wrong:** Compilation fails with `pkg-config` errors about missing libadwaita version.
**Why it happens:** Using `gnome_50` feature requires libadwaita 1.9+, but system has 1.8.4.
**How to avoid:** Use `features = ["libadwaita", "gnome_49"]` which enables `adw/v1_8`, matching system libadwaita 1.8.4. [VERIFIED: system pkg-config]
**Warning signs:** `cargo build` fails with version errors in glib/gtk bindings.

### Pitfall 4: SourceRegistry lifetime issues
**What goes wrong:** Cannot hold a `Source` reference across await points because `SourceRegistry::get()` returns `Option<&dyn Source>` with a lifetime tied to the registry.
**Why it happens:** The `dyn Source` trait object is borrowed from the registry's `Vec`, so the borrow cannot live across `.await` points in async code.
**How to avoid:** Two approaches: (1) use `Arc<dyn Source>` in the registry instead of `Box`, or (2) clone the source reference and collect needed data (source ID) before the await. The simplest fix is to refactor `SourceRegistry` to use `Arc<dyn Source>`, or to extract the source ID first and then call `registry.get(id)` after the await. Alternatively, spawn the fetch on a separate task via `sender.oneshot_command()` where the borrow lifetime is self-contained.
**Warning signs:** "borrowed value does not live long enough" errors at `.await` points.

### Pitfall 5: Cargo.toml workspace member not added
**What goes wrong:** `cargo build` doesn't compile the new `app` crate.
**Why it happens:** New `app/` directory must be added to workspace `members` in root `Cargo.toml`.
**How to avoid:** First task of the phase must add `"app"` to `workspace.members` in root `Cargo.toml`.
**Warning signs:** `cargo build` only compiles `domain`; `cargo build -p app` fails with "not found in workspace".

### Pitfall 6: relm4 edition and MSRV mismatch
**What goes wrong:** Compilation fails with edition or MSRV errors.
**Why it happens:** relm4 0.11.0 requires Rust edition 2024 and MSRV 1.93. [VERIFIED: crates.io API]
**How to avoid:** The `app/Cargo.toml` must specify `edition = "2024"`. System Rust is 1.94.1 which satisfies the MSRV. [VERIFIED: rustc --version]

## Code Examples

### Application Entry Point

```rust
// app/src/main.rs
use relm4::RelmApp;

mod app;
mod messages;
mod model;
mod widgets;

fn main() {
    let app = RelmApp::new("com.damask.Wallpaper");
    app.run::<app::AppComponent>(());
}
```

### Loading an Image into gtk::Picture from Cached Path

```rust
// Load wallpaper image from cache path
use gio::prelude::*;
use gtk::gdk::{self, Texture};
use gtk::prelude::*;

fn load_image(path: &std::path::Path, picture: &gtk::Picture) {
    let file = gio::File::for_path(path);
    match Texture::from_file(&file) {
        Ok(texture) => picture.set_paintable(Some(&texture)),
        Err(e) => eprintln!("Failed to load image: {}", e),
    }
}
```

### Populating Source Dropdown from Registry

```rust
// Create StringList from registry
use gtk::StringList;

let source_names: Vec<glib::GString> = registry.list()
    .iter()
    .map(|id| {
        registry.get(id).map(|s| s.name().into()).unwrap_or_default()
    })
    .collect();

let string_list = StringList::new(&source_names.iter()
    .map(|s| s.as_str())
    .collect::<Vec<_>>());
combo_row.set_model(Some(&string_list));
```

### Error Toast from Domain Error

```rust
// Domain errors implement Display, so they work directly as toast messages
fn handle_error<E: std::fmt::Display>(overlay: &adw::ToastOverlay, error: E) {
    let toast = adw::Toast::new(&error.to_string());
    toast.set_timeout(3);
    overlay.add_toast(&toast);
}

// Usage with any domain error:
// handle_error(&overlay, source_error);  // SourceError
// handle_error(&overlay, cache_error);   // CacheError
// handle_error(&overlay, de_error);      // DEError
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Component` + `Command` pattern | `AsyncComponent` trait | relm4 0.8+ | AsyncComponent is now the recommended default; cleaner async ergonomics |
| `gtk::Window` | `adw::ApplicationWindow` | libadwaita 1.0+ | ApplicationWindow provides theming, desktop integration |
| `gtk::MessageDialog` for errors | `adw::ToastOverlay` + `adw::Toast` | libadwaita 1.0+ | Non-blocking, matches GNOME HIG, accessible |
| Manual CSS for loading states | `init_loading_widgets()` | relm4 0.6+ | Built-in spinner during async init |

**Deprecated/outdated:**
- `relm4::Component` + `relm4::Command`: Still works but `AsyncComponent` is the modern replacement with better ergonomics.
- `gtk::ComboBox`: Use `adw::ComboRow` for libadwaita apps.
- `gtk::Dialog` for settings: Use `adw::PreferencesWindow`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `AsyncComponent` is strictly better than `Component` + `Commands` for this use case and is compatible with D-03's intent | Architecture Patterns | Low risk -- both work, AsyncComponent is newer. D-03 says "Commands" but the intent is async bridging, which AsyncComponent provides. |
| A2 | `init_loading_widgets()` is the correct API for showing a spinner during async init in relm4 0.11 | Architecture Patterns | Medium risk -- API could differ in 0.11 vs training data. Verify against relm4 docs at implementation time. |
| A3 | `adw::ComboRow::set_model()` accepts `gtk::StringList` | Code Examples | Low risk -- standard GTK4 ListModel pattern. If not, use `gtk::DropDown` as fallback. |
| A4 | relm4 manages its own tokio runtime, so domain async calls work without explicit runtime setup | Architecture Patterns | Medium risk -- relm4 uses glib async, not tokio directly. Domain uses tokio. May need `tokio::task::spawn_blocking()` bridge. Verify at implementation. |
| A5 | `gtk::Texture::from_file()` works with local file paths from the cache | Code Examples | Low risk -- standard GTK pattern for loading images from disk. |

## Open Questions

1. **relm4 runtime and tokio interop**
   - What we know: relm4 uses glib async internals. Domain crate uses tokio (with `tokio::fs`, `tokio::time::sleep`).
   - What's unclear: Whether relm4's runtime includes a tokio runtime or if domain's tokio calls will fail when called from within relm4's update loop.
   - Recommendation: Test early in the first task. If tokio calls panic inside relm4, use `tokio::runtime::Runtime::new()` in the app model and spawn domain tasks via `runtime.spawn()`. This is a common pattern in GTK Rust apps that mix glib and tokio.

2. **SourceRegistry borrow lifetime across async**
   - What we know: `SourceRegistry::get()` returns `Option<&dyn Source>` tied to `&self`.
   - What's unclear: Whether this causes issues with `sender.oneshot_command()` closures that need to borrow the registry.
   - Recommendation: If borrow issues arise, the simplest fix is to clone source IDs and call `get()` inside the command closure (requires the registry to be `Arc<Mutex<SourceRegistry>>` or similar). Alternatively, refactor domain's SourceRegistry to store `Arc<dyn Source>`.

3. **gio dependency in app crate**
   - What we know: gio is already a workspace dependency (0.20). `gio::File` is needed for loading images from cache paths.
   - What's unclear: Whether `gtk4` already re-exports gio, or if an explicit `gio = { workspace = true }` dependency is needed in app/Cargo.toml.
   - Recommendation: Add explicit `gio = { workspace = true }` to app/Cargo.toml. If it causes version conflicts, use the gtk4 re-export.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust compiler | Compilation | Yes | 1.94.1 (edition 2024) | -- |
| GTK4 | gtk4-rs bindings | Yes | 4.20.3 | -- |
| libadwaita | ToastOverlay, PreferencesWindow | Yes | 1.8.4 | -- |
| pkg-config | GTK/libadwaita detection | Yes | 2.3.0 | -- |
| tokio runtime | Domain async operations | Yes (via domain) | 1.51 (workspace) | -- |
| gio | File loading for Picture | Yes | system (via gtk4) | -- |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

All system libraries are present and meet version requirements.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None -- project constraint |
| Config file | None |
| Quick run command | `cargo build -p app` (compilation check only) |
| Full suite command | `cargo build -p app` (compilation check only) |

### Phase Requirements -> Test Map

Per project constraints ("No test coverage in v1 per user constraints; prioritize working MVP"), automated tests are not written for v1. Validation is manual verification against success criteria.

| Req ID | Behavior | Test Type | Verification | Wave 0? |
|--------|----------|-----------|--------------|---------|
| UI-01 | Wallpaper preview displays (300x200px) | Manual | Launch app, refresh, verify preview shows image | N/A |
| UI-02 | Metadata (title, desc, attribution) visible | Manual | Verify labels below preview show wallpaper metadata | N/A |
| UI-03 | Manual refresh fetches new wallpaper | Manual | Click refresh, observe new image and metadata | N/A |
| UI-04 | Source selector dropdown changes source | Manual | Select "Spotlight", refresh, verify Spotlight wallpaper | N/A |
| UI-05 | Error toasts shown on failure | Manual | Disconnect network, click refresh, verify toast appears | N/A |
| UI-06 | Settings window configures source options | Manual | Open settings, change Bing market, verify applied on refresh | N/A |

### Sampling Rate
- **Per task commit:** `cargo build -p app` (ensures compilation)
- **Per wave merge:** `cargo build -p app && cargo clippy -p app` (compilation + lint)
- **Phase gate:** Manual launch test against all 6 success criteria

### Wave 0 Gaps
None -- no test infrastructure needed per project constraints. All verification is manual.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A -- no user auth |
| V3 Session Management | No | N/A -- desktop app, no sessions |
| V4 Access Control | No | N/A -- single-user desktop app |
| V5 Input Validation | Yes (low) | Settings entry validation: Bing market format (xx-XX), Spotlight locale (numeric). Not critical -- invalid values cause API errors caught by domain. |
| V6 Cryptography | No | N/A -- no encryption, no secrets |

### Known Threat Patterns for GTK4 Desktop App

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious image URL from API response | Spoofing/Information Disclosure | `gtk::Picture` renders via GDK-Pixbuf which has built-in protections against malformed images. Domain's `validate_and_detect()` adds an additional layer. |
| File path traversal in cache | Tampering | Cache uses `wallpaper.id` as filename (controlled by domain, not user input). Cache directory is `~/.cache/damask/` -- standard XDG location. |
| No relevant threats | -- | This is a read-only wallpaper display app. No user input beyond dropdown selection and settings strings. No network exposure (outbound HTTP only). |

**Security assessment:** This phase has minimal security surface. The app makes outbound HTTPS requests to known APIs and writes images to `~/.cache/damask/`. No user-supplied content is rendered or executed. The domain crate already validates image formats (CVE protection via `image` crate).

## Sources

### Primary (HIGH confidence)
- [crates.io API: relm4 0.11.0](https://crates.io/api/v1/crates/relm4/0.11.0) -- Version, features, MSRV, edition, dependencies
- [crates.io API: gtk4-rs 0.11.2](https://crates.io/api/v1/crates/gtk4/0.11.2) -- Version, dependencies
- [crates.io API: libadwaita 0.9.1](https://crates.io/api/v1/crates/libadwaita/0.9.1) -- Version, dependencies
- [relm4.org book](https://relm4.org/book/stable/) -- AsyncComponent trait, view! macro, Commands
- [System pkg-config] -- GTK4 4.20.3, libadwaita 1.8.4
- [System rustc] -- Rust 1.94.1

### Secondary (MEDIUM confidence)
- [relm4 GitHub repository](https://github.com/Relm4/Relm4) -- Active development, issue tracker
- [gtk4-rs documentation](https://gtk-rs.org/gtk4-rs/) -- Widget API reference
- [libadwaita documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/) -- ToastOverlay, PreferencesWindow API

### Tertiary (LOW confidence)
- relm4 + tokio runtime interop -- Assumption A4, based on training knowledge. Verify at implementation time.
- `init_loading_widgets()` exact API signature -- Assumption A2, based on relm4 docs. Verify at implementation time.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- All versions verified via crates.io API; system libraries verified via pkg-config
- Architecture: HIGH -- AsyncComponent pattern verified via relm4 documentation; widget choices match libadwaita API
- Pitfalls: HIGH -- thumbnail_url gap verified via source code inspection; feature flag mismatch verified via system pkg-config
- Async interop: MEDIUM -- relm4/tokio boundary is the main uncertainty (Open Question 1)

**Research date:** 2026-04-14
**Valid until:** 30 days (stable crate ecosystem, no fast-moving dependencies)
