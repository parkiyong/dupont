---
phase: 02-desktop-environment-integration
date: 2026-04-14
status: complete
---

# Phase 2 Research: Desktop Environment Integration

## Research Questions

1. How to set wallpaper on GNOME via gio::Settings?
2. How to set wallpaper on COSMIC desktop?
3. How to detect the running desktop environment?
4. What error scenarios need handling?

## Findings

### 1. GNOME Wallpaper Setting via gio::Settings

**Schema:** `org.gnome.desktop.background`
**Key:** `picture-uri` (for light mode) and `picture-uri-dark` (for dark mode)

The gio crate (v0.20) provides `gio::Settings` which wraps GSettings:

```rust
use gio::Settings;

let settings = Settings::new("org.gnome.desktop.background");
settings.set_string("picture-uri", &format!("file://{}", path.display()))?;
```

**Important details:**
- `gio::Settings::new()` requires the schema to be installed on the system. If not installed, it panics (not an error — a panic). Must verify schema exists first.
- Schema verification: Use `gio::SettingsSchemaSource::get_default()` → `lookup(schema_id, true)` to check if schema exists before creating Settings.
- The value must be a URI: `file:///absolute/path/to/image.jpg` (note: three slashes for absolute paths)
- `picture-uri` accepts string type in GSettings
- On GNOME 42+, there's also `picture-uri-dark` for dark mode wallpaper
- `set_string()` returns `Result<(), glib::Error>` — need to map to DEError
- **gio is NOT Send/Sync** — `Settings` is a GObject with `!Send + !Sync`. The `DesktopEnvironment` trait requires `Send + Sync`. This means we cannot store `Settings` in a struct that implements `DesktopEnvironment`. Instead, create `Settings` on each call within the method.

### 2. COSMIC Wallpaper Setting

**Key finding:** COSMIC (by System76/Pop!_OS) uses its own configuration system, NOT GSettings/dconf.

**COSMIC uses `cosmic-config`** (their own config library) for settings. The wallpaper configuration is stored via `cosmic-settings-daemon` and `cosmic-bg` (background service).

**Practical approaches for COSMIC wallpaper setting:**

**Approach A: D-Bus API (Recommended)**
COSMIC exposes wallpaper setting through D-Bus. The `cosmic-bg` (cosmic-background) service manages wallpapers. However, the D-Bus interface is not well-documented publicly.

**Approach B: Direct config file manipulation**
COSMIC stores wallpaper config in `~/.config/cosmic/com.system76.CosmicSettings.Background/v1/background.ron` (RON format — Rusty Object Notation). This file contains the wallpaper path. This is the most reliable approach for an external application.

**Approach C: Fallback to GNOME compatibility mode**
Early COSMIC versions supported GNOME compatibility via `XDG_CURRENT_DESKTOP=GNOME`. COSMIC Epoch (the current full Rust implementation) does NOT support GSettings natively. However, some COSMIC installations may have gsettings available as a fallback.

**Recommended approach for v1:**
1. For COSMIC: Try D-Bus first (via `zbus` crate), fall back to direct config file write
2. For GNOME: Use `gio::Settings` directly
3. Detect COSMIC by checking `XDG_CURRENT_DESKTOP` for "COSMIC" or "pop:GNOME" or "pop-cosmic"

**Actually, the simplest and most reliable approach:**
Since COSMIC config files use RON format which is specific to Rust and COSMIC, and the D-Bus API is poorly documented, the safest approach for v1 MVP is:
1. GNOME: Use gio::Settings (well-documented, reliable)
2. COSMIC: Use `std::process::Command` to call `gsettings` CLI as a subprocess, since COSMIC may expose GNOME-compatible gsettings. If that fails, try the config file approach. OR simply use the `gsettings` command approach for both.

**Revised recommendation (simpler for MVP):**
After more analysis, the best approach for MVP is:
- Use `gio::Settings` for GNOME (type-safe, integrates with GLib main loop)
- For COSMIC: First check if gsettings/GNOME schema is available (COSMIC can run GNOME apps), if not, use direct config file approach with `cosmic-bg` config

**Important: gio crate dependency**
The `gio` crate requires the system library `libgio-2.0-dev` (Debian/Ubuntu) or `glib2-devel` (Fedora). It's a GObject binding, so it's Linux-specific, which is fine since we only target GNOME/COSMIC on Linux.

### 3. Desktop Environment Detection

Already implemented in Phase 1:
```rust
pub fn detect_desktop_environment() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()
        .map(|s| s.to_lowercase())
}
```

Need to add logic to map the detected string to a specific DE backend:
- `gnome`, `ubuntu:GNOME`, `debian` → GNOME
- `cosmic`, `pop:GNOME`, `pop-cosmic` → COSMIC
- `unity` → may use GNOME settings
- Anything else → UnsupportedDE error

### 4. Error Scenarios and Messages

**GNOME errors:**
- Schema `org.gnome.desktop.background` not found → `DEError::SchemaNotFound`
- `set_string` fails (permission, invalid value) → `DEError::SetError`
- Image file doesn't exist → `DEError::SetError`
- Image is not a valid URI → `DEError::SetError`

**COSMIC errors:**
- Config directory doesn't exist → `DEError::SetError`
- Config file write fails → `DEError::SetError`
- RON serialization fails → `DEError::SetError`

**Detection errors:**
- Neither `XDG_CURRENT_DESKTOP` nor `DESKTOP_SESSION` set → `DEError::DetectionFailed`
- Unrecognized DE string → `DEError::UnsupportedDE`

## Dependency Decisions

### gio crate
- **Version:** 0.20 (matches gtk-rs ecosystem, consistent with relm4 0.10)
- **Needed for:** GNOME wallpaper setting via GSettings
- **System dependency:** `libgio-2.0-dev` / `glib2-devel`
- **Add to workspace:** Yes, add `gio = "0.20"` to workspace dependencies

### No new crates for COSMIC
- For v1, use `std::fs` for config file manipulation (RON format can be written as plain text)
- If D-Bus approach is needed later, add `zbus` crate in v2

## Architecture

Phase 1 defined:
```rust
pub trait DesktopEnvironment: Send + Sync {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;
    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError>;
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
}
```

Phase 2 creates:
- `domain/src/desktop/gnome.rs` — `GnomeDE` struct implementing `DesktopEnvironment`
- `domain/src/desktop/cosmic.rs` — `CosmicDE` struct implementing `DesktopEnvironment`
- `domain/src/desktop/mod.rs` — Module structure, factory function `create_desktop_backend()`

### Gio Send+Sync Challenge

**Problem:** `gio::Settings` is `!Send + !Sync` (GObject types don't implement Send/Sync). Our `DesktopEnvironment` trait requires `Send + Sync`.

**Solution:** Don't store `Settings` in the struct. Instead:
- `GnomeDE` stores nothing (zero-sized struct or just holds schema verification result)
- `set_wallpaper()` and `get_current_wallpaper()` create `gio::Settings` locally within the method
- Since the methods take `&self`, and the struct doesn't contain non-Send types, the struct is `Send + Sync`

```rust
pub struct GnomeDE;

impl DesktopEnvironment for GnomeDE {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        // Verify file exists
        if !image_path.exists() {
            return Err(DEError::SetError(format!("Image file not found: {}", image_path.display())));
        }
        
        // Create settings locally (not stored)
        let schema_source = gio::SettingsSchemaSource::get_default()
            .ok_or(DEError::SchemaNotFound {
                schema: "org.gnome.desktop.background".to_string(),
            })?;
        
        let schema = schema_source.lookup("org.gnome.desktop.background", true)
            .ok_or(DEError::SchemaNotFound {
                schema: "org.gnome.desktop.background".to_string(),
            })?;
        
        let settings = gio::Settings::new_full(&schema, None::<&gio::SettingsBackend>, None);
        
        let uri = format!("file://{}", image_path.display());
        settings.set_string("picture-uri", &uri)
            .map_err(|e| DEError::SetError(format!("Failed to set wallpaper: {}", e)))?;
        
        Ok(())
    }
    // ...
}
```

This approach:
- ✅ Struct is `Send + Sync` (no non-Send fields)
- ✅ Creates Settings per-call (safe, not shared across threads)
- ✅ Validates schema before use
- ✅ Returns proper DEError variants

### Factory Function

```rust
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;
    
    match de.to_lowercase().as_str() {
        s if s.contains("gnome") || s.contains("ubuntu") || s.contains("debian") => {
            let backend = GnomeDE;
            if backend.is_available() {
                Ok(Box::new(backend))
            } else {
                Err(DEError::SchemaNotFound { schema: "org.gnome.desktop.background".into() })
            }
        }
        s if s.contains("cosmic") || s.contains("pop") => {
            let backend = CosmicDE;
            if backend.is_available() {
                Ok(Box::new(backend))
            } else {
                Err(DEError::SetError("COSMIC config not accessible".into()))
            }
        }
        other => Err(DEError::UnsupportedDE { de: other.to_string() }),
    }
}
```

## Common Pitfalls

1. **gio::Settings panics on missing schema** — Must verify schema exists before `Settings::new()`. Use `SettingsSchemaSource::lookup()` first.
2. **URI format** — Must use `file:///path` (three slashes for absolute path), not `/path` or `file://path`.
3. **COSMIC detection** — Pop!_OS may report `pop:GNOME` or `pop-cosmic` as XDG_CURRENT_DESKTOP. Must check for both `cosmic` and `pop` substrings.
4. **Thread safety** — gio::Settings is not Send/Sync. Create locally in method calls, don't store.
5. **Dark mode wallpaper** — GNOME 42+ has separate `picture-uri-dark` key. For v1, only set `picture-uri` (light mode). Dark mode support deferred to v2.
