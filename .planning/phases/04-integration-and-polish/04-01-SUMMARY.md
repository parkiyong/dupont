---
plan: 04-01
phase: 04-integration-and-polish
status: complete
completed: 2026-04-19
---

# Phase 4 Plan 1 Summary: Config Persistence

## Objective
Persist user preferences (Bing market, Spotlight locale, active source) to disk so they survive app restarts.

## Implementation

### Files Modified/Created
- `app/src/config.rs` (created) — Config struct with load/save, uses serde + dirs
- `app/src/main.rs` — Added `mod config`
- `app/src/app.rs` — Config loaded on init, saved on SettingsChanged/SourceChanged
- `app/Cargo.toml` — Added serde, serde_json, dirs, anyhow from workspace deps

### Key Changes
1. `Config` struct with `bing_market`, `spotlight_locale`, `active_source` fields
2. `Config::load()` reads from `~/.config/damask/config.json`, falls back to defaults on missing/corrupt file
3. `Config::save()` writes pretty-printed JSON, creates parent dirs if needed
4. App model reads config on startup, sets dropdown to match saved source
5. Settings changes and source changes persist immediately

### Build Status
- `cargo build -p damask-app` compiles (only pre-existing PreferencesWindow deprecation warnings)

## Verification
- Compiles without errors
- Config module handles missing/corrupt files gracefully (no panics)
- Config saves on settings and source changes
- Config loads and applies on startup

## Notes
- No config migration system — v1 first format
- Sync I/O for config (fast, negligible impact)
