# Dupont

## What This Is

Dupont is a Rust desktop wallpaper application for Linux desktop environments. It automatically fetches and sets desktop wallpapers from various online sources. The initial release targets GNOME and COSMIC desktop environments, starting with Microsoft Bing Wallpaper of the Day and Microsoft Spotlight as the supported wallpaper sources.

## Core Value

Users can automatically set their desktop wallpaper from online sources (Bing, Spotlight) with a simple, native Linux application.

## Requirements

### Validated

- ✓ Core engine can fetch wallpapers from Bing Wallpaper of the Day API — Validated in Phase 1
- ✓ Core engine can fetch wallpapers from Microsoft Spotlight API — Validated in Phase 1
- ✓ Core engine uses async operations (reqwest + tokio) to prevent UI blocking — Validated in Phase 1
- ✓ Core engine and UI layer are cleanly separated (easy to swap UI toolkit) — Validated in Phase 1
- ✓ Core engine can set wallpaper on GNOME desktop — Validated in Phase 2
- ✓ Core engine can set wallpaper on COSMIC desktop — Validated in Phase 2
- ✓ GTK-rs UI displays current wallpaper preview — Validated in Phase 3
- ✓ GTK-rs UI allows manual refresh of wallpaper — Validated in Phase 3
- ✓ GTK-rs UI allows selecting wallpaper source (Bing/Spotlight) — Validated in Phase 3
- ✓ GTK-rs UI displays wallpaper metadata (title, description, attribution) — Validated in Phase 3
- ✓ GTK-rs UI displays error toasts for failures — Validated in Phase 3
- ✓ GTK-rs UI provides settings window for source configuration — Validated in Phase 3
- ✓ Application runs complete end-to-end workflow — Validated in Phase 4
- ✓ Configuration persists source preferences across sessions — Validated in Phase 4
- ✓ Application includes desktop file for app menu integration — Validated in Phase 4
- ✓ Application includes appropriate icon — Validated in Phase 4
- ✓ Flatpak package with sandbox permissions — Validated in Phase 5
- ✓ Arch Linux AUR package (PKGBUILD) — Validated in Phase 5

### Active

(None — all v1 requirements validated)

### Out of Scope

- [All other wallpaper sources] — Wallhaven, NASA APOD, Unsplash, EarthView, local files defer to v2
- [Automatic refresh] — Manual refresh only for MVP
- [Flatpak packaging] — Local development only for MVP
- [Other desktop environments] — Only GNOME and COSMIC

## Context

Dupont is a learning project to gain experience with Rust GUI development while building a useful desktop application. Inspired by the Damask wallpaper application (written in Vala/GTK). The project emphasizes clean architecture with a clear separation between the core wallpaper engine (fetching, caching, setting backgrounds) and the UI layer, enabling future portability to other UI toolkits like Iced or web-based frontends. As a Rust GUI beginner, the focus is on getting working software quickly while establishing good patterns for future expansion.

## Constraints

- **UI Toolkit**: GTK-rs for v1 — Must be easily swappable to Iced or other toolkits later via clean architecture
- **Desktop Environments**: GNOME and COSMIC only — Must handle wallpaper setting APIs for both DEs
- **Timeline**: Weeks — MVP must be working quickly, prioritize completion over features
- **Sources**: Only Bing and Spotlight — Original Damask has 8+ sources, defer all others
- **Testing**: No test coverage in v1 — Skip unit tests to focus on getting software working
- **Learning Focus**: Beginner-friendly code structure — Avoid over-optimizations, prioritize clarity
- **Scope**: Minimal MVP — Basic fetch and set wallpaper functionality with simple UI, no advanced features

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| GTK-rs for v1 | Mature bindings, native GNOME experience | — Pending |
| Clean engine/UI separation | Enables future UI toolkit swap (Iced for COSMIC), better architecture | ✓ Good |
| Bing + Spotlight only | Reduces complexity for MVP, two most reliable API sources | — Pending |
| Manual refresh only | Automatic refresh adds complexity (timers, background services) | — Pending |
| No tests in v1 | Prioritize working MVP over test coverage for learning project | — Pending |
| GNOME + COSMIC support | User runs both DEs, wallpaper setting APIs differ | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-19 after Phase 5 (Packaging) — v1.1 milestone complete*
