# Phase 3: UI Layer Implementation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 03-ui-layer-implementation
**Areas discussed:** UI framework choice, window layout, async integration, error handling UX

---

## UI Framework Choice

| Option | Description | Selected |
|--------|-------------|----------|
| relm4 | Component-based framework on gtk4-rs. Component trait with init/update, message passing, built-in async via Commands. Mature, well-documented. | ✓ |
| raw gtk4-rs | Direct gtk4-rs bindings. Full control but no component model or message passing. More boilerplate. | |
| You decide | Leave choice to planner based on simplicity. | |

**User's choice:** relm4
**Notes:** User selected recommended option. relm4 is the most natural fit for GTK4 in Rust with async support.

---

## Window Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Preview-focused | Large preview on top, metadata below, controls at bottom. Image is main content. | ✓ |
| Sidebar + preview | Sidebar with controls on left, preview on right. More structured. | |
| Floating controls | Minimal floating controls overlaid on full-window preview. Sleeker but more complex. | |

**User's choice:** Preview-focused
**Notes:** Natural for a wallpaper app where the image is the main content.

---

## Async Integration

| Option | Description | Selected |
|--------|-------------|----------|
| relm4 Commands | Component::update dispatches Commands (async futures) sending messages back. Standard relm4 pattern. | ✓ |
| Manual tokio channels | Spawn tokio tasks with channels for communication. More control but more boilerplate. | |

**User's choice:** relm4 Commands
**Notes:** Leverages relm4's built-in async support rather than fighting the framework.

---

## Error Handling UX

| Option | Description | Selected |
|--------|-------------|----------|
| Toast overlay | adw::ToastOverlay wraps content. Temporary toasts at bottom, auto-dismiss. Matches GNOME HIG. | ✓ |
| Inline error labels | Error messages shown inline near relevant widget. More contextual but needs layout space. | |
| Both depending on type | Toasts for network errors, inline for set errors. More targeted but adds complexity. | |

**User's choice:** Toast overlay
**Notes:** Clean, non-blocking, matches GNOME HIG via libadwaita.

---

## Claude's Discretion

- Exact widget sizing and spacing
- Loading state visual (spinner vs skeleton)
- Toast duration and detail level
- Settings window layout and widget organization
- Metadata display formatting

## Deferred Ideas

None — discussion stayed within phase scope
