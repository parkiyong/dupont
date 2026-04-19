# Changelog

All notable changes to Dupont are documented in this file.

## [1.2.0] - 2026-04-19

### Added
- Wallpaper Portal backend via ashpd (org.freedesktop.portal.Wallpaper) for Flatpak compatibility
- AppStream metadata for software center integration
- Rust 2024 edition for modern language features

### Changed
- Binary renamed from `dupont-app` to `dupont` for simpler command-line invocation
- Updated all packaging (AUR PKGBUILD, Flatpak manifest, .desktop file) to reflect new binary name
- Upgraded Rust edition from 2021 to 2024 for both domain and app crates
- Removed unnecessary filesystem permissions from Flatpak manifest (uses sandboxed XDG dirs)

### Technical
- domain/Cargo.toml: 0.1.0 → 1.2.0
- app/Cargo.toml: 0.1.0 → 1.2.0
- Both crates now use Rust 2024 edition

## [1.1.0] - 2026-04-15

### Added
- Initial stable release
- Bing Wallpaper of the Day support (configurable market)
- Microsoft Spotlight support (configurable locale)
- COSMIC desktop environment backend (direct RON config file)
- LRU image cache (~500MB, 50 images, 30 days retention)
- GTK4/relm4-based UI with settings dialog
- Persistent user configuration (market, locale, active source)
- Desktop integration (desktop file, app icons scalable + symbolic)
- AUR packaging support
- Flatpak packaging (GNOME 49 runtime)

### Features
- Fetch wallpapers from online sources
- Preview with metadata (title, description, attribution)
- Manual refresh trigger
- Source switching (Bing ↔ Spotlight)
- Error toasts for network/runtime failures
- XDG Base Directory Specification compliance

---

**Note:** Versions before 1.1 were development versions. See [GitHub releases](https://github.com/parkiyong/dupont/releases) for full history.
