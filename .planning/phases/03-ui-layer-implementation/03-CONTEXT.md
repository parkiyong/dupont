# Phase 3: UI Layer Implementation - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

GTK4 desktop application that displays wallpaper preview with metadata, source selector dropdown, manual refresh button, error toasts, and a settings window. This is the presentation layer only — all business logic lives in the existing `domain` crate.

</domain>

<decisions>
## Implementation Decisions

### UI Framework
- **D-01:** Use relm4 as the component framework on top of gtk4-rs — Component trait with init/update pattern, message passing, built-in async support via Commands

### Window Layout
- **D-02:** Preview-focused layout — large wallpaper preview fills most of the window (main content), metadata displayed below the preview, source selector and refresh button at the bottom as controls

### Async Integration
- **D-03:** Use relm4's Command system for async operations — update() dispatches Commands that send messages back when done, loading state tracked via component model field

### Error Handling UX
- **D-04:** Use adw::ToastOverlay wrapping main content — errors shown as temporary toast messages at the bottom that auto-dismiss, non-blocking, matches GNOME HIG

### Claude's Discretion
- Exact widget sizing and spacing
- Loading state visual (spinner vs skeleton)
- Toast duration and detail level
- Settings window layout and widget organization
- Metadata display formatting

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Domain crate API
- `domain/src/lib.rs` — Public re-exports of all domain types (Wallpaper, Source, SourceRegistry, Cache, DesktopEnvironment, etc.)
- `domain/src/wallpaper.rs` — Wallpaper struct fields: id, url, title, description, attribution, source, thumbnail_url
- `domain/src/source.rs` — Source trait (fetch, id, name) and SourceRegistry (register, get, list)
- `domain/src/error.rs` — SourceError, CacheError, DEError variants for toast messages
- `domain/src/desktop/mod.rs` — DesktopEnvironment trait and create_desktop_backend() factory
- `domain/src/cache.rs` — Cache and CacheConfig for wallpaper image caching

### Project constraints
- `.planning/PROJECT.md` — GTK-rs for v1, GNOME and COSMIC only, manual refresh only, no tests in v1
- `.planning/ROADMAP.md` Phase 3 section — UI-01 through UI-06 requirements and success criteria

### External
- relm4 documentation — https://relm4.org/ for Component trait, Commands, and widget patterns

No other external specs — requirements are fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `domain::SourceRegistry` — Pre-built registry to manage Bing/Spotlight sources; UI calls `register()` at startup, `list()` for dropdown, `get()` for fetch
- `domain::Wallpaper` — Ready-to-use struct with title, description, attribution, url, thumbnail_url for metadata display
- `domain::create_desktop_backend()` — Single factory call returns Box<dyn DesktopEnvironment>; UI just calls `set_wallpaper(path)` on the result
- `domain::Cache` — Handles image download and caching with LRU eviction; UI gets cached file path for preview

### Established Patterns
- Domain crate is fully independent of UI — all types are Send + Sync, no GTK dependencies
- Error types use thiserror with descriptive messages — directly usable for toast content
- Async via async_trait + tokio — relm4 Commands will bridge to tokio runtime

### Integration Points
- New `app` crate (workspace member) will depend on `domain` + `relm4` + `gtk4` + `libadwaita`
- Cargo.toml workspace needs new member entry
- UI component model messages map to domain operations: fetch wallpaper, set wallpaper, switch source, open settings

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches that match GNOME HIG and libadwaita conventions.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-ui-layer-implementation*
*Context gathered: 2026-04-14*
