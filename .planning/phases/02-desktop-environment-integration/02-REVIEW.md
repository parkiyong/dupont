---
phase: 02-desktop-environment-integration
reviewed: 2026-04-14T12:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - Cargo.toml
  - domain/Cargo.toml
  - domain/src/desktop/cosmic.rs
  - domain/src/desktop/gnome.rs
  - domain/src/desktop/mod.rs
  - domain/src/lib.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-14T12:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Reviewed the desktop environment integration layer: trait definition (`DesktopEnvironment`), GNOME backend (via gio::Settings), COSMIC backend (via direct RON config file), detection/factory logic, and workspace Cargo.toml changes. The architecture is clean -- a trait with per-DE implementations behind a factory function. However, there are two critical bugs: an RON config injection vulnerability in the COSMIC backend and a double-lowercase call in detection logic. There are also warnings around incomplete URI handling in GNOME and a subtle string-processing edge case in COSMIC parsing.

## Critical Issues

### CR-01: RON config injection via crafted image path (COSMIC backend)

**File:** `domain/src/desktop/cosmic.rs:56-62`
**Issue:** The `write_wallpaper_config` method interpolates `image_path.to_string_lossy()` directly into a RON template string without any escaping or sanitization. If the image path contains a double-quote (`"`), newline, or RON control characters, it will break the RON structure or allow injection of arbitrary RON content. Since the file is written to `~/.config/cosmic/...` and presumably read by `cosmic-bg`, this could cause unexpected behavior in the COSMIC desktop session.

Example: a path like `/tmp/wall"paper\nother_field: Some(0)` would produce malformed or malicious RON.

```rust
// VULNERABLE: raw interpolation into RON template
let content = format!(
    r#"Some(Wallpaper {{
    path: Some("{}"),
    color: None,
}})"#,
    path_str  // unescaped user-controlled path
);
```

**Fix:** Escape double-quotes and backslashes in the path string before interpolation:

```rust
fn write_wallpaper_config(image_path: &Path) -> Result<(), DEError> {
    let config_dir = Self::ensure_config_dir()?;
    let config_path = config_dir.join(COSMIC_WALLPAPER_FILE);

    let path_str = image_path.to_string_lossy();
    // Escape characters that are special in RON string literals
    let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");

    let content = format!(
        r#"Some(Wallpaper {{
    path: Some("{}"),
    color: None,
}}"#,
        escaped
    );

    fs::write(&config_path, content).map_err(|e| {
        DEError::SetError(format!(
            "Failed to write COSMIC wallpaper config to {}: {}",
            config_path.display(),
            e
        ))
    })?;

    Ok(())
}
```

### CR-02: Double `.to_lowercase()` in `create_desktop_backend` -- dead code path

**File:** `domain/src/desktop/mod.rs:38-39`
**Issue:** `detect_desktop_environment()` already calls `.to_lowercase()` on line 31 before returning. Then `create_desktop_backend()` calls `.to_lowercase()` **again** on line 39, producing `de_lower` from the already-lowercased string. This is not just redundant -- it indicates a copy-paste oversight. More critically, the variable `de` on line 38 (the already-lowercased string from detection) is passed directly to `DEError::UnsupportedDE { de }` on line 72, which means the error message will show a lowercased DE name (e.g., "gnome" instead of "GNOME"). This loses the original case and makes the error message less useful for debugging.

```rust
let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;
let de_lower = de.to_lowercase(); // de is ALREADY lowercase
```

**Fix:** Either remove `.to_lowercase()` from `detect_desktop_environment()` and keep it only in `create_desktop_backend()`, or (simpler) just remove the redundant call:

```rust
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;
    // de is already lowercased by detect_desktop_environment()

    if de.contains("cosmic") {
        let backend = CosmicDE;
        if backend.is_available() {
            return Ok(Box::new(backend));
        }
        let gnome = GnomeDE;
        if gnome.is_available() {
            return Ok(Box::new(gnome));
        }
    }
    // ... use `de` directly instead of `de_lower`
}
```

If preserving the original case in error messages is desired, modify `detect_desktop_environment` to return the raw string and lowercase only in the caller.

## Warnings

### WR-01: Incorrect file:// URI construction for paths with spaces or special characters

**File:** `domain/src/desktop/gnome.rs:50-52`
**Issue:** `path_to_uri` naively prepends `"file://"` to the display-formatted path. Per RFC 8089, file URIs must percent-encode reserved characters (spaces, `#`, `?`, etc.). A wallpaper at `/home/user/My Photos/wall.jpg` would produce `file:///home/user/My Photos/wall.jpg` which is technically invalid and may not work correctly with all GSettings consumers. GNOME's GSettings itself may tolerate this, but it is fragile.

```rust
fn path_to_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}
```

**Fix:** Use `glib::filename_to_uri()` or `gio::File` which handles percent-encoding correctly:

```rust
fn path_to_uri(path: &Path) -> Result<String, DEError> {
    let gfile = gio::File::for_path(path);
    gfile.get_uri().ok_or_else(|| {
        DEError::SetError("Failed to convert path to URI".to_string())
    })
}
```

This also requires updating the call site in `set_wallpaper` to handle the `Result`. Alternatively, `glib::filename_to_uri(path, None)` returns `Result<String, glib::Error>`.

### WR-02: COSMIC `get_current_wallpaper` parsing is fragile and incorrect for escaped quotes

**File:** `domain/src/desktop/cosmic.rs:105-111`
**Issue:** The parser looks for `path: Some("` and then finds the next `")`. If CR-01 is fixed (escaping quotes), this parser will break because it does not handle escaped quotes (`\"`). The path `"C:\\Users\\test\"wall"` would truncate at the escaped quote. Additionally, the parser does not validate that what it extracted is a valid file path.

```rust
if let Some(start) = content.find("path: Some(\"") {
    let path_start = start + "path: Some(\"".len();
    if let Some(end) = content[path_start..].find("\")") {
        let path_str = &content[path_start..path_start + end];
        return Ok(Some(PathBuf::from(path_str)));
    }
}
```

**Fix:** If CR-01 fix is applied (escaping quotes on write), update the read parser to handle escaped quotes. Alternatively, use the `ron` crate for proper serialization/deserialization instead of manual string manipulation:

```rust
// Option A: handle escaped quotes
if let Some(start) = content.find("path: Some(\"") {
    let path_start = start + "path: Some(\"".len();
    let remaining = &content[path_start..];
    let mut result = String::new();
    let mut chars = remaining.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(escaped);
            }
        } else if c == '"' {
            break;
        } else {
            result.push(c);
        }
    }
    if !result.is_empty() {
        return Ok(Some(PathBuf::from(result)));
    }
}
```

### WR-03: `XDG_CURRENT_DESKTOP` can contain colon-separated list -- partial handling

**File:** `domain/src/desktop/mod.rs:26-31`
**Issue:** Per the freedesktop.org specification, `XDG_CURRENT_DESKTOP` may contain a colon-separated list of desktop names (e.g., `ubuntu:GNOME` or `pop:GNOME`). The detection function treats the entire string as a single DE name and lowercases it. The subsequent `.contains()` checks in `create_desktop_backend` happen to work for cases like `ubuntu:gnome` (because `contains("gnome")` matches the substring), but this is fragile. A value like `sway:GNOME` would match `gnome` even though sway is a Wayland compositor, not GNOME.

```rust
pub fn detect_desktop_environment() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()
        .map(|s| s.to_lowercase())
}
```

**Fix:** Split on `:` and use the first entry as the primary DE, or iterate through all entries to find the best match:

```rust
pub fn detect_desktop_environment() -> Option<String> {
    let raw = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()?;

    // XDG_CURRENT_DESKTOP can be colon-separated; use the first entry
    let primary = raw.split(':').next()?.trim();
    Some(primary.to_lowercase())
}
```

## Info

### IN-01: Match arm simplification in `get_current_wallpaper` (GNOME)

**File:** `domain/src/desktop/gnome.rs:96-99`
**Issue:** The `match` on `path` is unnecessary -- both arms return `Ok(Some(p))` / `Ok(None)` which is equivalent to `Ok(path)`.

```rust
let path = Self::uri_to_path(uri_str);
match path {
    Some(p) => Ok(Some(p)),
    None => Ok(None),
}
```

**Fix:**
```rust
let path = Self::uri_to_path(uri_str);
Ok(path)
```

### IN-02: Workspace `gio` dependency is declared but only used by the `domain` crate

**File:** `Cargo.toml:15`
**Issue:** The `gio = "0.20"` workspace dependency is only consumed by `domain/Cargo.toml`. This is not a bug -- workspace-level declarations for single-crate usage are a valid organizational pattern. Noting it purely for awareness; if the workspace grows and other crates do not need `gio`, it could be moved to `domain/Cargo.toml` directly.

---

_Reviewed: 2026-04-14T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
