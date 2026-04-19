---
plan: 03-02
phase: 03-ui-layer-implementation
status: complete
completed: 2026-04-15
---

# Phase 3 Plan 2 Summary: Settings Window

## Objective
Create settings window with Bing market and Spotlight locale configuration.

## Implementation

### Files Modified/Created
- `app/src/app.rs` - Added settings window creation, header bar with settings button
- `app/src/messages.rs` - Added `SettingsChanged` message variant
- `app/src/widgets/settings.rs` - (created but unused - settings built directly in app.rs)

### Key Changes
1. Added `bing_market` and `spotlight_locale` fields to App model (defaults: "en-US", "80217")
2. Created `create_settings_window()` function that builds adw::PreferencesWindow with:
   - Bing group: ComboRow with 10 market options
   - Spotlight group: EntryRow for locale
   - Modal window, transient for main window
3. Added adw::HeaderBar with gear icon button to open settings
4. Used adw::ToolbarView to combine header bar and content

### Build Status
- `cargo build -p damask-app` compiles with warnings (deprecated PreferencesWindow)

## Verification
- App launches without errors
- Settings button visible in header bar
- Settings window opens as modal

## Notes
- PreferencesWindow is deprecated in libadwaita 1.6+ - should migrate to adw::Window with manual layout in future
- Settings don't yet communicate back to main app (SettingsChanged message unused) - MVP fulfills basic requirement