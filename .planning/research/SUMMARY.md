# Project Research Summary

**Project:** damask-rs — Rust desktop wallpaper application (Linux)
**Domain:** Desktop application / wallpaper manager
**Researched:** 2026-04-13
**Confidence:** HIGH

## Executive Summary

Damask-rs is a Linux desktop wallpaper application that fetches wallpapers from online sources (Bing, Microsoft Spotlight) and sets them as the desktop background. This is a Rust rewrite of Damask (originally written in Vala), targeting GNOME and COSMIC desktop environments with manual refresh functionality for MVP. The application type is a desktop utility with I/O-heavy operations (HTTP requests, image downloading, desktop API calls) where UI responsiveness is critical.

Experts build this type of application by separating core engine logic from UI, using async patterns for all I/O to keep the UI responsive, and abstracting desktop environment APIs to enable multi-platform support. Research strongly recommends a clean trait-based architecture (`WallpaperEngine`, `Source`, `DesktopEnvironment`) with gtk4-rs + relm4 for the UI layer, enabling future migration to Iced for COSMIC-native UI without rewriting core logic. Key risks include blocking the GTK main thread (UI freezes), assuming single desktop environment behavior (breaks on COSMIC), and cache bloat (no cleanup strategy). These are mitigated by using async from day one (reqwest + tokio), implementing runtime DE detection, and designing cache with LRU eviction from start.

## Key Findings

### Recommended Stack

Research identifies a mature Rust ecosystem for GTK development with strong async support. gtk4-rs is battle-tested (56+ production apps on gtk-rs.org), relm4 accelerates development with Elm-inspired patterns, and reqwest + tokio provide the foundation for non-blocking I/O. The stack balances performance, maintainability, and extensibility while adhering to Linux desktop standards (XDG directories, GSettings).

**Core technologies:**
- **Rust 1.93+**: Core language with memory safety and performance; minimum version required by relm4 0.10+
- **gtk4 0.11 + relm4 0.10**: GUI framework (bindings) with Elm-inspired UI patterns; mature, native GNOME experience, excellent async support with ComponentSender
- **reqwest 0.12 + tokio 1.51**: Async HTTP client and runtime; non-blocking network operations to prevent UI freezes
- **gio 0.20**: Desktop integration (gsettings) for wallpaper setting on GNOME/COSMIC; type-safe API integration with GLib main loop
- **dirs 5.0**: XDG-compliant directory paths; follows Linux standards (~/.cache, ~/.config, ~/.local/share)
- **thiserror 2.0 + anyhow 1.0**: Domain-specific and application error handling; structured error types for engine, easy propagation in UI

### Expected Features

Feature research analyzed competitor landscape (Damask, Picture of the Day, WonderWall) and identified table stakes vs. differentiators. Manual refresh, source selection, and wallpaper preview are non-negotiable — users expect these in any wallpaper app. The research explicitly defers anti-features (video wallpapers, advanced gallery, multi-monitor) to avoid scope creep and maintain focus on "fetch and set" value proposition.

**Table stakes (must have for MVP):**
- **Manual refresh button** — users expect on-demand wallpaper changes (<5s, non-blocking)
- **Source selection (Bing, Spotlight)** — users expect choice between wallpaper sources
- **Wallpaper preview** — users want to see what they're getting before it's set
- **GNOME + COSMIC wallpaper setting** — core functionality; different APIs for each DE
- **Error handling with toasts** — silent failures frustrate users; all errors must be visible
- **Settings window** — standard GTK pattern for source-specific configuration
- **Basic caching** — avoid re-downloading; simple directory-based cache (cleanup in v1.x)

**Differentiators (should have):**
- **Rust implementation** — memory safety, performance, modern tooling vs. Vala (Damask)
- **Async/non-blocking operations** — never freezes UI; smooth user experience
- **Clean engine/UI architecture** — enables future Iced migration for COSMIC-native UI
- **Robust error handling** — clear, actionable error messages; no silent failures
- **Proper caching with LRU eviction** — prevents disk bloat; differentiator vs. competitors who cache indefinitely
- **Runtime DE detection** — works seamlessly on GNOME and COSMIC without user configuration

**Defer to v2+:**
- **Multi-monitor support** — low early adoption usage; HydraPaper exists for this niche
- **Iced UI migration** — requires clean engine architecture first (do this before adding sources!)
- **Advanced search/filtering** — WonderWall does this well; different app category (gallery vs automatic)
- **Automatic refresh timer** — manual-only for MVP; add in v1.x if users request it
- **Extensive source catalog** — start with 2 proven sources; add more in v1.x after validation

### Architecture Approach

Research recommends a layered architecture with strict separation between core engine (domain logic) and UI layer (GTK/relm4). The core engine uses trait-based abstractions (`Source`, `DesktopEnvironment`, `Cache`) to enable plugin-like extensibility and future UI toolkit swaps. This pattern is critical for the project constraint: "Must be easily swappable to Iced or other toolkits later via clean architecture."

**Major components:**
1. **Core Engine (domain crate)** — domain logic, wallpaper orchestration, fetch → cache → set workflow; no UI dependencies
2. **Sources (trait-based plugin system)** — wallpaper source implementations (Bing, Spotlight); extendable via `Source` trait
3. **Cache Manager** — download, store, retrieve images; XDG-compliant cache directory with LRU eviction
4. **DE API Adapter (DesktopEnvironment trait)** — set wallpaper on desktop; separate implementations for GNOME (GSettings) and COSMIC
5. **UI Layer (gtk4-rs + relm4)** — GTK widgets, user interaction, display; observes core engine state, handles signals

### Critical Pitfalls

Pitfalls research identified 5 critical failures common in this domain, with clear prevention strategies. The most insidious are silent failures (GSettings schema assumptions work on GNOME but break COSMIC) and cache bloat (indefinite growth to gigabytes). All critical pitfalls map to specific phases for verification.

1. **Blocking GTK main thread with I/O** — use async HTTP clients (reqwest), spawn heavy operations with `glib::spawn_future_local`, never call `.await` directly in signal handlers
2. **Cross-desktop environment compatibility assumptions** — detect DE at runtime (`XDG_CURRENT_DESKTOP`), abstract wallpaper setting behind `DesktopEnvironment` trait, test on both GNOME and COSMIC before considering features complete
3. **Image caching without cleanup strategy** — design cache schema before implementing (max size 500MB, max age 30 days, max count 50), implement LRU eviction from day one, expose cache stats in UI
4. **GSettings schema path assumptions** — verify schema exists at runtime, fall back to alternative paths, document which GNOME versions are tested (GNOME 45+)
5. **Image format assumptions without robust decoding** — handle all `image::ImageError` variants explicitly, pre-verify images before caching, set image library limits to prevent OOM

## Implications for Roadmap

Based on research, suggested phase structure follows architecture boundaries: core engine first (testable without UI), then desktop integration (requires real DE), then UI layer (depends on everything else). This order prevents the most common pitfalls (main thread blocking, DE assumptions) by establishing async patterns and abstractions early.

### Phase 1: Core Engine Foundation
**Rationale:** Core engine can be tested without UI complexity; establishes async patterns to prevent main thread blocking (Pitfall 1); implements cache schema to prevent bloat (Pitfall 3); creates trait abstractions for clean architecture.
**Delivers:** Source trait, WallpaperEngine trait, DesktopEnvironment trait stubs, basic cache with LRU eviction, async HTTP infrastructure (reqwest + tokio)
**Addresses:** Table stakes features (manual refresh flow, basic caching, Bing/Spotlight source implementations)
**Avoids:** Main thread blocking (async from day 1), cache bloat (cleanup strategy designed upfront), UI in domain layer (strict separation)

### Phase 2: Desktop Environment Integration
**Rationale:** Requires real GNOME/COSMIC environment for testing; implements DE-specific wallpaper setting APIs; adds runtime DE detection to prevent assumptions (Pitfall 2); verifies GSettings schema paths (Pitfall 4).
**Delivers:** GNOME backend (GSettings wrapper), COSMIC backend (research required, likely cosmic-settings D-Bus API), DE detection logic, schema verification, fallback mechanisms
**Uses:** gio 0.20 (GSettings API), DesktopEnvironment trait (from Phase 1)
**Implements:** Architecture component (DE API Adapter)
**Avoids:** Cross-DE compatibility assumptions (runtime detection, trait abstraction), GSettings schema errors (verification on startup)

### Phase 3: UI Layer Implementation
**Rationale:** UI depends on core engine and DE integration; gtk4-rs + relm4 provide mature patterns; implements user-facing features (preview, settings, toasts).
**Delivers:** GTK application structure (gtk::Application), main window widgets (preview, refresh button, source selector), settings window, error toasts, signal handlers wired to core engine
**Uses:** gtk4 0.11 + relm4 0.10 (UI framework), glib::spawn_future_local (async in UI), core engine (Phase 1), DE backends (Phase 2)
**Addresses:** Table stakes features (wallpaper preview, settings window, error handling with toasts, source selection dropdown)
**Avoids:** Blocking I/O on UI thread (all operations use core engine's async methods), inconsistent error handling (centralized toast notifications)

### Phase 4: Integration and Polish
**Rationale:** Wire everything together; add missing pieces (configuration loading, application lifecycle); verify end-to-end workflows; prepare for release.
**Delivers:** Application entry point, configuration management (YAML/settings), integration tests, documentation, packaging (desktop files, icons)
**Uses:** dirs 5.0 (XDG paths for config), thiserror + anyhow (error handling across crates), all previous phases
**Addresses:** Differentiators (runtime DE detection, robust error handling), table stakes (run in background toggle optional)
**Avoids:** UX pitfalls (progress indication, error messages, preview before setting)

### Phase Ordering Rationale

- **Core first** enables testing without UI complexity; establishes async patterns and trait abstractions that prevent critical pitfalls (main thread blocking, cache bloat, DE assumptions)
- **DE integration second** requires real desktop environment for testing; validates architecture abstraction before UI complexity obscures issues; catches GSettings schema errors early
- **UI layer third** depends on stable core engine and DE integration; GTK/relm4 patterns are well-documented, making this phase straightforward if foundation is solid
- **Integration last** brings everything together; this is where user-facing bugs emerge (missing error messages, progress indicators); final polish before release

**Architecture-based grouping:**
- Phase 1: Core engine (domain crate) — pure Rust, testable
- Phase 2: Desktop backends (domain/desktops) — DE-specific, requires real environment
- Phase 3: UI layer (ui crate) — GTK widgets, user interaction
- Phase 4: Application (main.rs) — wiring everything together

**Pitfall avoidance by order:**
- Main thread blocking prevented by Phase 1 (async patterns established before UI)
- Cross-DE assumptions prevented by Phase 2 (DE detection and abstraction before UI hardcodes paths)
- Cache bloat prevented by Phase 1 (cleanup strategy designed before caching implemented)
- GSettings errors prevented by Phase 2 (schema verification before UI shows user errors)

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (COSMIC backend):** COSMIC desktop wallpaper API is poorly documented; requires phase-specific research to find correct D-Bus API or mechanism
- **Phase 3 (GTK4/relm4 patterns):** While gtk-rs is mature, best practices for async in relm4 signals vary; review official examples and relm4 docs during planning

Phases with standard patterns (skip research-phase):
- **Phase 1 (Core engine):** Well-documented patterns (async, traits, caching); reqwest/tokio have extensive examples
- **Phase 4 (Integration):** Standard Rust patterns (testing, packaging); XDG and desktop file specs are mature

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technologies verified with official docs (gtk-rs.org, docs.rs), crates.io for current versions, gtk-rs.org app examples for production proof |
| Features | MEDIUM | Competitor analysis based on Flathub app pages (MEDIUM confidence), Damask source code (HIGH confidence), feature landscape well-established in domain |
| Architecture | HIGH | Patterns verified with gtk-rs official docs, relm4.org, original Damask Vala codebase; trait-based abstractions are standard Rust practice |
| Pitfalls | HIGH | Pitfalls verified with gtk-rs docs, GSettings API docs, common patterns in Linux desktop apps; all pitfalls have concrete prevention strategies |

**Overall confidence:** HIGH

Research is strong across all areas. Stack research is HIGH confidence (official sources, production examples). Architecture patterns are HIGH confidence (well-established in Rust ecosystem). Pitfalls are HIGH confidence (verified with official docs and common patterns). Feature research is MEDIUM confidence (competitor landscape from Flathub, but feature requirements are standard for this domain).

### Gaps to Address

Minor gaps identified during research that need validation during planning:

- **COSMIC wallpaper API:** Research found sparse documentation for COSMIC's wallpaper setting mechanism. GNOME uses GSettings, but COSMIC is newer and likely uses cosmic-settings D-Bus API or direct file writing. Plan Phase 2 with research step to verify correct approach.
- **Relm4 async integration:** While relm4 supports async via ComponentSender, specific patterns for spawning futures in signal handlers vary. Review relm4 examples during Phase 3 planning to ensure correct async usage (avoid main thread blocking).
- **Image format edge cases:** Bing/Spotlight APIs typically return JPEG/PNG, but may return WebP or HEIC. Research recommends robust image decoding with format detection, but specific edge cases need testing during Phase 1 implementation.
- **Flatpak sandboxing:** Research identified potential issues with cache directory access in Flatpak sandbox. Plan to verify correct Flatpak permissions (`--filesystem=xdg-cache/damask-rs`) during Phase 4 packaging.

None of these gaps are showstoppers; they can be addressed during phase planning with targeted research or spike solutions.

## Sources

### Primary (HIGH confidence)
- [gtk-rs.org](https://gtk-rs.org) — GTK4 installation, available crates, ecosystem overview, 56+ production app examples
- [relm4.org](https://relm4.org) — Relm4 documentation, async patterns, ComponentSender API, tokio integration
- [docs.rs/crates/reqwest](https://docs.rs/reqwest) — Reqwest API documentation, features, async patterns
- [docs.rs/crates/image](https://docs.rs/image) — Image crate documentation, format handling, error variants
- [crates.io](https://crates.io) — Current stable versions: gtk4 0.11, relm4 0.10, reqwest 0.12, tokio 1.51, dirs 5.0
- [Damask source code](https://gitlab.gnome.org/subpop/damask) — Original Vala implementation, analyzed directly for architecture patterns
- [GNOME developer documentation](https://developer.gnome.org/gio/stable/gio-GSettings.html) — GSettings API for wallpaper setting

### Secondary (MEDIUM confidence)
- [Flathub wallpaper apps collection](https://flathub.org/apps/collection/tag/wallpaper) — Competitor landscape, feature analysis
- [Flathub Damask page](https://flathub.org/apps/details/app.drey.Damask) — Feature list, user-facing description
- [Flathub HydraPaper page](https://flathub.org/apps/details/org.gabmus.hydrapaper) — Multi-monitor niche, anti-feature validation
- [Flathub WonderWall page](https://flathub.org/apps/details/com.ktechpit.wonderwall) — Gallery features, scope creep reference
- [Flathub Picture of the Day page](https://flathub.org/apps/details/de.swsnr.pictureoftheday) — Multi-source patterns
- [Flatpak common issues](https://github.com/flatpak/flatpak/issues) — Sandbox permissions, filesystem access
- [Flathub wiki](https://github.com/flathub/flathub/wiki) — App requirements, packaging guidelines

### Tertiary (LOW confidence)
- [System76 COSMIC documentation](https://docs.system76.com) — COSMIC desktop integration; specific wallpaper API not well-documented, needs phase-specific research
- [COSMIC panel issues](https://github.com/pop-os/cosmic-panel/issues) — COSMIC-specific API discussions; sparse documentation, community-reported issues

---
*Research completed: 2026-04-13*
*Ready for roadmap: yes*
