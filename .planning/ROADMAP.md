# Roadmap: Damask-rs

**Defined:** 2026-04-13
**Core Value:** Users can automatically set their desktop wallpaper from online sources (Bing, Spotlight) with a simple, native Linux application.

## Phases

- [x] **Phase 1: Core Engine Foundation** - Core domain logic with async operations and caching (completed 2026-04-14)
- [x] **Phase 2: Desktop Environment Integration** - GNOME and COSMIC wallpaper setting APIs (completed 2026-04-14)
- [ ] **Phase 3: UI Layer Implementation** - GTK widgets for user interaction
- [ ] **Phase 4: Integration and Polish** - Application wiring and packaging

## Phase Details

### Phase 1: Core Engine Foundation
**Goal**: Core domain logic implements trait-based architecture with async fetching from Bing and Spotlight APIs
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06
**Success Criteria** (what must be TRUE):
  1. Core engine fetches wallpapers from Bing Wallpaper of the Day API successfully
  2. Core engine fetches wallpapers from Microsoft Spotlight API successfully
  3. All I/O operations use async (reqwest + tokio) without blocking the main thread
  4. Core engine implements clean architecture with trait-based separation (Source, DesktopEnvironment traits)
  5. Core engine caches downloaded wallpapers with LRU eviction to prevent disk bloat (max 500MB, 50 images, 30 days)
**Plans**: 3 plans

**Plan List:**
- [x] 01-01-PLAN.md — Establish workspace structure and core trait definitions
- [x] 01-02-PLAN.md — Implement Bing and Spotlight wallpaper sources with async fetching
- [x] 01-03-PLAN.md — Implement image caching with LRU eviction and format validation

### Phase 2: Desktop Environment Integration
**Goal**: Application detects runtime DE and sets wallpaper on GNOME and COSMIC with clear error messages
**Depends on**: Phase 1
**Requirements**: DESK-01, DESK-02, DESK-03, DESK-04
**Success Criteria** (what must be TRUE):
  1. Application detects runtime desktop environment (GNOME vs COSMIC) via XDG_CURRENT_DESKTOP
  2. Application sets wallpaper on GNOME desktop using gio::Settings with schema verification
  3. Application sets wallpaper on COSMIC desktop using appropriate API (cosmic-settings D-Bus or GNOME compatibility)
  4. Application provides clear error messages for all wallpaper setting failures (GSettings errors, missing schemas, DE detection issues)
**Plans**: 2 plans

**Plan List:**
- [x] 02-01-PLAN.md — Add gio dependency, restructure desktop module, implement GNOME backend
- [x] 02-02-PLAN.md — Implement COSMIC backend and wire into factory function

### Phase 3: UI Layer Implementation
**Goal**: GTK application displays wallpaper preview, source selector, and refresh button with error toasts
**Depends on**: Phase 1, Phase 2
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, UI-06
**Success Criteria** (what must be TRUE):
  1. GTK application displays current wallpaper preview (300x200px thumbnail)
  2. GTK application displays wallpaper metadata (title, description, attribution) in preview
  3. GTK application allows manual refresh of wallpaper via button
  4. GTK application allows selecting wallpaper source via dropdown (Bing, Spotlight)
  5. GTK application displays error toasts for all failures (network, decode, set errors)
  6. GTK application provides settings window for source-specific configuration (Bing market, Spotlight locale)
**Plans**: 2 plans
**UI hint**: yes

**Plan List:**
- [x] 03-01-PLAN.md — Create app crate with main window, preview, metadata, source dropdown, refresh, and error toasts
- [ ] 03-02-PLAN.md — Create settings window with Bing market and Spotlight locale configuration

### Phase 4: Integration and Polish
**Goal**: Application integrates all components with configuration loading and prepares for release
**Depends on**: Phase 1, Phase 2, Phase 3
**Requirements**: (None - integration phase)
**Success Criteria** (what must be TRUE):
  1. Application runs complete workflow: launch → select source → refresh → set wallpaper
  2. Configuration persists source preferences (Bing market, Spotlight locale) across sessions
  3. Application includes desktop file for integration with GNOME/COSMIC application menus
  4. Application includes appropriate icon for desktop environment display
**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Engine Foundation | 3/3 | Complete    | 2026-04-14 |
| 2. Desktop Environment Integration | 2/2 | Complete    | 2026-04-14 |
| 3. UI Layer Implementation | 0/2 | Planning    | - |
| 4. Integration and Polish | 0/0 | Not started | - |

---
*Roadmap created: 2026-04-13*
