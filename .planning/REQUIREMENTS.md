# Requirements: Damask-rs

**Defined:** 2026-04-13
**Core Value:** Users can automatically set their desktop wallpaper from online sources (Bing, Spotlight) with a simple, native Linux application.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Core Engine

- [ ] **CORE-01**: Core engine can fetch wallpapers from Bing Wallpaper of the Day API
- [ ] **CORE-02**: Core engine can fetch wallpapers from Microsoft Spotlight API
- [ ] **CORE-03**: Core engine uses async operations (reqwest + tokio) to prevent UI blocking
- [ ] **CORE-04**: Core engine implements clean architecture with trait-based separation (WallpaperEngine trait)
- [ ] **CORE-05**: Core engine caches downloaded wallpapers with LRU eviction to prevent disk bloat
- [ ] **CORE-06**: Core engine validates image formats (WebP, HEIC) with robust error handling

### Desktop Integration

- [ ] **DESK-01**: Application detects runtime desktop environment (GNOME vs COSMIC) via XDG_CURRENT_DESKTOP
- [ ] **DESK-02**: Application sets wallpaper on GNOME desktop using gio::Settings with schema verification
- [ ] **DESK-03**: Application sets wallpaper on COSMIC desktop (implementation TBD, may use GNOME compatibility or D-Bus)
- [ ] **DESK-04**: Application provides clear error messages for all wallpaper setting failures (GSettings errors, missing schemas, DE detection issues)

### User Interface

- [ ] **UI-01**: GTK-rs UI displays current wallpaper preview (300x200px thumbnail)
- [ ] **UI-02**: GTK-rs UI displays wallpaper metadata (title, description, attribution) in preview
- [ ] **UI-03**: GTK-rs UI allows manual refresh of wallpaper via button
- [ ] **UI-04**: GTK-rs UI allows selecting wallpaper source via dropdown (Bing, Spotlight)
- [ ] **UI-05**: GTK-rs UI displays error toasts for all failures (network, decode, set errors)
- [ ] **UI-06**: GTK-rs UI provides settings window for source-specific configuration (Bing market, Spotlight locale)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Additional Features

- **CORE-07**: Core engine supports additional sources (NASA APOD, Unsplash, Wallhaven)
- **UI-07**: GTK-rs UI supports automatic refresh timer (configurable interval)
- **UI-08**: GTK-rs UI provides background mode toggle (run without window)
- **UI-09**: GTK-rs UI allows opening wallpaper source URL in browser
- **DESK-05**: Application sets different wallpapers per monitor (multi-monitor support)

### COSMIC Native

- **UI-10**: Iced UI implementation for COSMIC-native experience (requires UI toolkit swap)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| All other Damask sources (Wallhaven, NASA APOD, Unsplash, EarthView, local files) | Defer to v2; Bing and Spotlight sufficient for MVP |
| Automatic refresh timer | Adds background service complexity; manual refresh only for MVP |
| Background mode / autostart | Conflicts with "manual refresh only" constraint; defer to v1.x |
| Multi-monitor different wallpapers | Complex image merging, low early adoption usage; defer to v2 |
| Video wallpapers | Different app category (Hidamari), resource-heavy; out of scope |
| Advanced gallery/browser | Already done well by WonderWall; scope creep, dilutes "automatic wallpaper" value |
| Image editing/cropping | Adds complexity; users expect photo editor features; out of scope |
| Social features (sharing, favorites) | Requires backend, user accounts, privacy concerns; out of scope |
| NSFW content support | Legal/compliance issues, violates safety norms; keep family-friendly |
| Flatpak packaging | Local development only for MVP; defer to v1.x |
| Unit tests | No test coverage in v1 per user constraints; prioritize working MVP |
| Other desktop environments | Only GNOME and COSMIC; other DEs (XFCE, KDE) out of scope |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Pending |
| CORE-02 | Phase 1 | Pending |
| CORE-03 | Phase 1 | Pending |
| CORE-04 | Phase 1 | Pending |
| CORE-05 | Phase 1 | Pending |
| CORE-06 | Phase 1 | Pending |
| DESK-01 | Phase 2 | Pending |
| DESK-02 | Phase 2 | Pending |
| DESK-03 | Phase 2 | Pending |
| DESK-04 | Phase 2 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 3 | Pending |
| UI-03 | Phase 3 | Pending |
| UI-04 | Phase 3 | Pending |
| UI-05 | Phase 3 | Pending |
| UI-06 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 16/16 ✓
- Unmapped: 0

---
*Requirements defined: 2026-04-13*
*Last updated: 2026-04-13 after initial definition*
