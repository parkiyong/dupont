# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Dupont is a native Linux desktop wallpaper app (Bing WOTD + Microsoft Spotlight) built with Rust, GTK4/libadwaita via relm4. Currently supports GNOME only.

## Build & Test Commands

```bash
cargo build --release                    # Full release build
cargo run --release -p dupont-app        # Run the app
cargo test                               # Run unit tests (in domain/src/error.rs, domain/src/desktop/mod.rs)
cargo clippy                             # Lint
cargo fmt --check                        # Format check
```

Flatpak build (requires GNOME 49 SDK):
```bash
cd flatpak && flatpak-builder --user --install --force-clean repo io.github.parkiyong.dupont.json
```

## Architecture

Two-crate Cargo workspace with clean domain/UI separation:

- **`domain/` (dupont-domain)** — business logic, no UI dependency
  - `source.rs` — `Source` trait (async `fetch()` → `Wallpaper`)
  - `sources/bing.rs`, `sources/spotlight.rs` — source implementations
  - `cache.rs` — LRU image cache (~500MB/50 images/30 days)
  - `desktop/mod.rs` — `DesktopEnvironment` trait + `create_desktop_backend()` factory (reads `XDG_CURRENT_DESKTOP`)
  - `desktop/portal.rs` — GNOME wallpaper via Portal API (ashpd/DBus)
  - `desktop/cosmic.rs` — COSMIC wallpaper via direct RON file write
  - `error.rs` — `SourceError`, `CacheError`, `DEError` (thiserror)
  - `wallpaper.rs` — `Wallpaper` struct (id, url, title, description, attribution, source)

- **`app/` (dupont-app)** — GTK4/relm4 UI
  - `app.rs` — single async root component; handles all state, message dispatch, and widget building
  - `config.rs` — user settings persistence (`~/.config/dupont/config.json`)
  - `messages.rs` — `AppMsg` enum (Refresh, SourceChanged, SettingsChanged)
  - `widgets/` — UI widget components (controls, preview, settings)

## Key Patterns

- **Async-first**: tokio runtime, reqwest HTTP, `Arc<tokio::sync::Mutex<Cache>>` for shared state
- **Trait-based extensibility**: add sources by implementing `Source`, add DEs by implementing `DesktopEnvironment` and updating the factory in `desktop/mod.rs`
- **Relm4 message flow**: user action → `AppMsg` → `update()` spawns `oneshot_command` → async fetch/cache → `CmdOut` → UI update
- **XDG compliance**: cache in `~/.cache/dupont/`, config in `~/.config/dupont/`
- **Shared mutable config**: `Rc<RefCell<String>>` for market/locale shared between app and settings window

## Planning

Project plans and requirements live in `.paul/` (PAUL framework). Check `STATE.md` for current status, `ROADMAP.md` for phases.
