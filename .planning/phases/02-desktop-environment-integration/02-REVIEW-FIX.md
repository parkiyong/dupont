---
phase: 02-desktop-environment-integration
fixed_at: 2026-04-14T12:30:00Z
review_path: .planning/phases/02-desktop-environment-integration/02-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 02: Code Review Fix Report

**Fixed at:** 2026-04-14T12:30:00Z
**Source review:** .planning/phases/02-desktop-environment-integration/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: RON config injection via crafted image path (COSMIC backend)

**Files modified:** `domain/src/desktop/cosmic.rs`
**Commit:** `35dc13f`
**Applied fix:** Added escaping of backslashes and double-quotes in the image path string before interpolating into the RON config template. The `write_wallpaper_config` method now calls `.replace('\\', "\\\\").replace('"', "\\\"")` on the path string to prevent RON injection via crafted file paths.

### CR-02: Double `.to_lowercase()` in `create_desktop_backend` -- dead code path

**Files modified:** `domain/src/desktop/mod.rs`
**Commit:** `ee8282a`
**Applied fix:** Removed the redundant `let de_lower = de.to_lowercase()` line and replaced all usages of `de_lower` with `de` directly in `create_desktop_backend()`. The string returned by `detect_desktop_environment()` is already lowercased, so the second call was unnecessary. The `de` variable is now used directly in all `.contains()` checks and the `UnsupportedDE` error message.

### WR-01: Incorrect file:// URI construction for paths with spaces or special characters

**Files modified:** `domain/src/desktop/gnome.rs`
**Commit:** `15a8e87`
**Applied fix:** Replaced naive `format!("file://{}", path.display())` with `gio::File::for_path(path).uri().into()` which handles proper percent-encoding per RFC 8089. Changed `path_to_uri` return type from `String` to `Result<String, DEError>`, added `FileExt` trait import, and updated the call site in `set_wallpaper` to propagate errors with `?`.

### WR-02: COSMIC `get_current_wallpaper` parsing is fragile and incorrect for escaped quotes

**Files modified:** `domain/src/desktop/cosmic.rs`
**Commit:** `07564cc`
**Applied fix:** Replaced the naive `find("\")")` parser with a character-by-character parser that handles escaped sequences. When it encounters `\`, it consumes the next character as a literal (unescaped) value. This correctly handles paths written by the CR-01 fix which escapes `\` as `\\` and `"` as `\"`.

### WR-03: `XDG_CURRENT_DESKTOP` can contain colon-separated list -- partial handling

**Files modified:** `domain/src/desktop/mod.rs`
**Commit:** `4c6b21a`
**Applied fix:** Updated `detect_desktop_environment()` to split the `XDG_CURRENT_DESKTOP` value on `:` and use only the first entry (per freedesktop.org specification). This prevents false matches like `sway:GNOME` incorrectly matching GNOME. The function now uses early-return style with `?` operator instead of `.ok().map()`.

---

_Fixed: 2026-04-14T12:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
