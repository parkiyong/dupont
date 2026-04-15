---
phase: 03-ui-layer-implementation
plan: 01
subsystem: ui
tags: [relm4, gtk4, libadwaita, async-component, tokio, tokio-mutex]

# Dependency graph
requires:
  - phase: 02-desktop-environment-integration
    provides: "domain crate with Source, Cache, DesktopEnvironment traits and implementations"
provides:
  - "Launchable damask-app binary with GTK4 window titled 'Damask'"
  - "Wallpaper preview via gtk::Picture with ContentFit::Cover"
  - "Metadata display (title, description, attribution) with CSS classes"
  - "Source selector dropdown (Bing Wallpaper of the Day, Microsoft Spotlight)"
  - "Refresh button with spinner during loading and suggested-action styling"
  - "Error toasts via adw::ToastOverlay with 5-second timeout"
  - "Async fetch flow via oneshot_command with CmdOut pattern"
  - "Automatic desktop wallpaper application on successful fetch"
affects: [04-integration-testing]

# Tech tracking
tech-stack:
  added: [relm4 0.11, gtk4 0.11, libadwaita 0.9]
  patterns:
    - "AsyncComponent with manual widget construction (no view! macro)"
    - "update_with_view for input handling with widget access"
    - "update_cmd_with_view for command output handling with widget access"
    - "tokio::sync::Mutex for Send-safe cache access across await"
    - "CmdOut enum pattern for background task results instead of sender.input() routing"
    - "Direct source instantiation in async closures (avoids borrow-across-await on SourceRegistry)"

key-files:
  created:
    - app/Cargo.toml
    - app/src/main.rs
    - app/src/app.rs
    - app/src/messages.rs
    - app/src/widgets/mod.rs
    - app/src/widgets/controls.rs
    - app/src/widgets/preview.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Used tokio::sync::Mutex instead of std::sync::Mutex to avoid Send trait error when holding MutexGuard across await"
  - "Manual widget construction instead of view! macro due to macro parsing limitations with adw::Clamp nesting"
  - "Overrode update_with_view instead of update to get widget access for loading state UI changes"
  - "Used CmdOut enum for command results instead of routing through sender.input() for cleaner separation"
  - "Stored widgets in separate Widgets struct returned from init() for access in update_cmd_with_view"
  - "Created fresh source instances in async closure instead of borrowing from SourceRegistry to avoid borrow-across-await"

patterns-established:
  - "AsyncComponent with manual widget tree construction and Widgets struct"
  - "CmdOut enum for background task result routing to update_cmd_with_view"
  - "Button child swapping (Label <-> Spinner) for loading state indication"

requirements-completed: [UI-01, UI-02, UI-03, UI-04, UI-05]

# Metrics
duration: 11min
completed: 2026-04-15
---

# Phase 3 Plan 01: Main Application Window Summary

**GTK4 application shell with wallpaper preview, metadata display, source selector, refresh with spinner, error toasts, and automatic desktop wallpaper setting using relm4 AsyncComponent**

## Performance

- **Duration:** 11 min
- **Started:** 2026-04-15T00:42:49Z
- **Completed:** 2026-04-15T00:53:37Z
- **Tasks:** 3 (2 auto, 1 auto-approved checkpoint)
- **Files modified:** 9

## Accomplishments
- Launchable `damask-app` binary with GTK4 window titled "Damask" (480x520)
- Wallpaper preview with gtk::Picture and ContentFit::Cover
- Metadata labels (title, description, attribution) with proper CSS classes (heading, body, caption)
- Source selector dropdown (Bing Wallpaper of the Day, Microsoft Spotlight) with signal wiring
- Refresh button with spinner animation during loading and suggested-action styling
- Error toasts via adw::ToastOverlay with 5-second auto-dismiss
- Async fetch flow using relm4 oneshot_command with CmdOut pattern
- Automatic desktop wallpaper application on successful fetch via create_desktop_backend()
- Empty state with placeholder text before first fetch

## Task Commits

Each task was committed atomically:

1. **Task 1: Create app crate with AsyncComponent scaffold, model, messages, and workspace integration** - `cb2ab1e` (feat)
2. **Task 2: Wire async domain calls, loading states, error toasts, and image preview loading** - `19b1001` (feat)
3. **Task 3: Verify main window launches, fetches wallpaper, displays preview and metadata, shows error toasts** - auto-approved (checkpoint)

## Files Created/Modified
- `Cargo.toml` - Added "app" to workspace members
- `Cargo.lock` - Updated with new dependencies
- `app/Cargo.toml` - App crate with relm4 0.11 + gtk4 + libadwaita (edition 2024)
- `app/src/main.rs` - Entry point with RelmApp::run_async
- `app/src/app.rs` - Main AppComponent with AsyncComponent, manual widget tree, async fetch, loading states, error toasts
- `app/src/messages.rs` - AppMsg enum (Refresh, SourceChanged)
- `app/src/widgets/mod.rs` - Widget module placeholder
- `app/src/widgets/controls.rs` - Controls placeholder
- `app/src/widgets/preview.rs` - Preview placeholder

## Decisions Made
- **tokio::sync::Mutex over std::sync::Mutex**: std::sync::MutexGuard is not Send, which caused the oneshot_command future to fail the Send bound when held across an await point. tokio::sync::MutexGuard is Send.
- **Manual widget construction over view! macro**: The relm4 view! macro had parsing issues with deeply nested adw::Clamp containers. Manual construction gave full control without macro limitations.
- **update_with_view override over update**: Needed widget access during input handling (AppMsg::Refresh) to set loading state UI (spinner, disabled dropdown).
- **CmdOut enum over sender.input() routing**: Cleaner separation between input messages (user actions) and command output (background task results). Avoids mixing concerns in the AppMsg enum.
- **Direct source instantiation over SourceRegistry borrow**: SourceRegistry::get() returns &dyn Source which borrows &self and cannot cross await. Creating fresh BingSource::new()/SpotlightSource::new() in the async closure avoids this entirely.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed domain package name in Cargo.toml**
- **Found during:** Task 1 (Create app crate)
- **Issue:** Plan specified `domain = { path = "../domain" }` but the crate's actual package name is `damask-domain`
- **Fix:** Changed to `domain = { path = "../domain", package = "damask-domain" }`
- **Files modified:** app/Cargo.toml
- **Verification:** cargo build -p damask-app succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed relm4 0.11 import paths**
- **Found during:** Task 1 (Create app crate)
- **Issue:** AsyncComponent, AsyncComponentParts, AsyncComponentSender not exported from relm4 root; need relm4::prelude::* or relm4::component::*
- **Fix:** Changed imports to use `relm4::prelude::*` and direct component module imports
- **Files modified:** app/src/app.rs
- **Verification:** Compilation succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**3. [Rule 3 - Blocking] Fixed std::sync::Mutex Send trait error**
- **Found during:** Task 1 (Create app crate)
- **Issue:** std::sync::MutexGuard is not Send, causing oneshot_command future to fail when held across cache.get_or_download().await
- **Fix:** Switched to tokio::sync::Mutex whose MutexGuard IS Send
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**4. [Rule 3 - Blocking] Fixed view! macro parsing failures with adw::Clamp**
- **Found during:** Task 1 (Create app crate)
- **Issue:** The relm4 view! macro could not parse the widget tree with adw::Clamp as a container, even with #[wrap(Some)] set_child syntax
- **Fix:** Replaced view! macro with manual widget construction using gtk::Builder pattern and direct append/set_child calls
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**5. [Rule 3 - Blocking] Fixed widget access in update() vs update_cmd_with_view()**
- **Found during:** Task 1 (Create app crate)
- **Issue:** update() does not receive widgets parameter; only update_cmd_with_view() and update_with_view() do
- **Fix:** Overrode update_with_view() instead of update() for input message handling to get widget access for loading state changes
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** 19b1001 (Task 2 commit)

**6. [Rule 3 - Blocking] Fixed AdwApplicationWindowExt trait not in scope**
- **Found during:** Task 1 (Create app crate)
- **Issue:** root.set_content() and root.content() require AdwApplicationWindowExt trait; adw::prelude::* provides it
- **Fix:** Added `use adw::prelude::*` import
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**7. [Rule 3 - Blocking] Fixed relm4::Sender::input() doesn't exist**
- **Found during:** Task 1 (Create app crate)
- **Issue:** input_sender() returns &Sender<T> which has send(), not input()
- **Fix:** Changed sender_refresh.input() to sender_refresh.send()
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** cb2ab1e (Task 1 commit)

**8. [Rule 3 - Blocking] Fixed gtk::Button::remove() not available**
- **Found during:** Task 2 (Wire async domain calls)
- **Issue:** gtk::Button is not a Container, so remove() method doesn't exist
- **Fix:** Use set_child(Some(&new_widget)) which replaces the child automatically
- **Files modified:** app/src/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** 19b1001 (Task 2 commit)

---

**Total deviations:** 8 auto-fixed (1 bug, 7 blocking)
**Impact on plan:** All auto-fixes were necessary to make the code compile with relm4 0.11's actual API. The core functionality and architecture remain as planned. The main architectural difference is manual widget construction instead of the view! macro.

## Issues Encountered
- relm4 0.11's view! macro had difficulty with adw::Clamp nested inside a gtk::Box inside adw::ToastOverlay. Multiple approaches tried (direct nesting, #[wrap] syntax) before switching to manual construction.
- relm4 0.11 exports AsyncComponent/AsyncComponentParts only from relm4::component::* or relm4::prelude::*, not from the root crate as the plan assumed.
- The domain crate's package name is `damask-domain` not `domain`, requiring a package rename in the dependency declaration.

## User Setup Required
None - no external service configuration required. The app connects to public Bing and Spotlight APIs with default settings.

## Known Stubs
- `app/src/widgets/controls.rs` - Empty placeholder for future controls widget extraction
- `app/src/widgets/preview.rs` - Empty placeholder for future preview widget extraction
- These are intentional placeholders per the plan (Step G) for potential future extraction.

## Next Phase Readiness
- All 5 UI requirements (UI-01 through UI-05) are implemented in this plan
- Plan 02 (Settings Window) can proceed - the App struct and component pattern are established
- The widgets/ module has placeholder files ready for potential extraction in future plans

---
*Phase: 03-ui-layer-implementation*
*Completed: 2026-04-15*

## Self-Check: PASSED

- All 8 created files verified present
- Both task commits verified (cb2ab1e, 19b1001)
- No unexpected file deletions in commits
- cargo build -p damask-app compiles cleanly
- cargo clippy -p damask-app passes with zero warnings