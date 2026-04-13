# Feature Research

**Domain:** Linux wallpaper application (automatic desktop wallpaper from online sources)
**Researched:** 2026-04-13
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Manual refresh button | Users expect to get a new wallpaper on demand | LOW | Single click operation, must be fast (<5s) and non-blocking UI |
| Source selection (dropdown) | Users expect to choose between wallpaper sources | LOW | Bing and Spotlight for MVP; other sources later |
| Wallpaper preview | Users want to see wallpaper before it's set | LOW | Thumbnail display (300x200px) with title/description/attribution |
| Set wallpaper to desktop | Core functionality - app must actually change wallpaper | MEDIUM | Different APIs for GNOME (GSettings) and COSMIC (cosmic-settings) |
| Error handling with user feedback | Silent failures frustrate users, prevent debugging | MEDIUM | Toast notifications for all errors (download, decode, set) |
| Settings/preferences window | Standard GTK/GNOME pattern for configuration | LOW | Source-specific settings (Bing market/resolution, Spotlight locale) |
| Run in background mode | Users expect wallpaper app to continue without window | LOW | Keep-alive mechanism (gtk::Application::hold()), optional for MVP |
| Download/caching | Avoid re-downloading same wallpapers | MEDIUM | Cache directory with XDG compliance, LRU eviction, size limits |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Rust implementation | Memory safety, performance, modern tooling | LOW | Differentiator vs Vala (Damask) and Electron apps; attracts Rust developers |
| Async/non-blocking operations | Never freezes UI, smooth user experience | MEDIUM | reqwest async + tokio + glib spawn_future_local; critical for perceived performance |
| Clean engine/UI architecture | Enables future UI toolkit swap (Iced for COSMIC) | HIGH | Trait-based abstraction (WallpaperEngine) separates concerns; long-term extensibility |
| Robust error handling | Clear, actionable error messages; no silent failures | MEDIUM | thiserror for domain errors + anyhow for app errors; better UX than competitors |
| Proper caching with cleanup | Prevents disk bloat, transparent resource usage | MEDIUM | Many competitors cache indefinitely; our LRU eviction is a differentiator |
| Runtime DE detection | Works seamlessly on GNOME and COSMIC | LOW | Detects `XDG_CURRENT_DESKTOP`, automatically selects correct wallpaper API |
| Progressive source support | Add new sources without UI rewrites | LOW | Plugin-like architecture via WallpaperSource trait; Bing+Spotlight first, more later |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Video wallpapers | "Dynamic wallpapers are cool" | Different app category (Hidamari), requires complex playback, resource-heavy | Use separate video wallpaper app; stay focused on static images |
| Advanced gallery/browser | "Browse thousands of wallpapers" | Already done well by WonderWall; scope creep, dilutes "automatic wallpaper" value proposition | Use WonderWall for browsing; our app = "set and forget" |
| Extensive source catalog (10+ sources) | "More options = better" | Adds maintenance burden, API changes break sources, dilutes focus | Start with 2 reliable sources (Bing, Spotlight); add proven sources in v2 |
| Automatic refresh timer (configurable) | "Get new wallpaper every hour automatically" | Background services, timers, app lifecycle complexity; conflicts with "manual refresh MVP" constraint | Manual only for MVP; add timer in v1.x if users request it |
| Multi-monitor different wallpapers | "Set different image on each screen" | Complex image merging (HydraPaper approach), cross-DE complexity, low usage in early adoption | Accept single monitor wallpaper for MVP; defer to v2 if user demand exists |
| Image editing/cropping | "Adjust wallpaper to fit screen" | Adds complexity to app scope; users expect photo editor features | Use system image tools; our app = "fetch and set", not "edit" |
| Social features (sharing, favorites) | "Share wallpapers with friends" | Requires backend, user accounts, privacy concerns; out of scope for MVP | Use existing social platforms; focus on local wallpaper management |
| NSFW content support | "Adult wallpapers available" | Legal/compliance issues, violates safety norms, harms app reputation | Block NSFW filters; keep app family-friendly by default |

## Feature Dependencies

```
[Wallpaper Preview]
    ├──requires──> [Image Caching]
    └──requires──> [Source Implementation (Bing/Spotlight)]
                    └──requires──> [HTTP Client (reqwest)]

[Manual Refresh]
    ├──requires──> [Source Implementation]
    ├──requires──> [Wallpaper Setting (GNOME/COSMIC)]
    └──enhances──> [Error Handling Toasts]

[Error Handling Toasts]
    └──requires──> [UI Framework (gtk4/relm4)]

[Background Mode]
    ├──requires──> [Application Lifecycle Management]
    └──conflicts──> [Manual-Only Operation] (if autostart enabled)

[Multiple DE Support]
    ├──requires──> [Runtime DE Detection]
    ├──requires──> [GNOME GSettings Integration]
    └──requires──> [COSMIC Settings Integration]

[Clean Engine/UI Architecture]
    └──enables──> [Future UI Toolkit Swap (Iced)]
```

### Dependency Notes

- **[Wallpaper Preview] requires [Image Caching]:** Thumbnails must be stored to avoid re-fetching; cache required for performance
- **[Manual Refresh] requires [Wallpaper Setting]:** Core functionality is fetch + set; setting API differs by DE, need abstraction
- **[Manual Refresh] enhances [Error Handling Toasts]:** Refresh is primary error trigger (network, decode, set failures)
- **[Background Mode] conflicts with [Manual-Only Operation]:** If autostart is enabled, app runs automatically; this conflicts with MVP "manual refresh only" constraint; defer background mode to v1.x
- **[Multiple DE Support] requires [Runtime DE Detection]:** Cannot hardcode DE-specific APIs; must detect `XDG_CURRENT_DESKTOP` at runtime
- **[Clean Engine/UI Architecture] enables [Future UI Toolkit Swap]:** Trait-based abstraction allows rewriting UI layer without touching core engine; critical for COSMIC Iced migration in v2

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] **Manual refresh** — Core value prop: users get new wallpaper on demand
- [ ] **Bing Wallpaper source** — Proven, reliable API; covers "online wallpaper" use case
- [ ] **Spotlight source** — Differentiates from "just Bing"; demonstrates source architecture
- [ ] **Wallpaper preview** — Users see what they're getting before it's set; prevents unwanted wallpapers
- [ ] **GNOME wallpaper setting** — Target DE 1: largest Linux desktop; validate fetch → set workflow
- [ ] **COSMIC wallpaper setting** — Target DE 2: user's actual DE; validate multi-DE support
- [ ] **Error handling with toasts** — Silent failures = broken UX; all errors must be visible
- [ ] **Settings window** — Source-specific configuration (Bing market, Spotlight locale)
- [ ] **Basic caching** — Avoid re-downloading; simple directory-based cache (no cleanup yet)

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **Cache cleanup strategy** — Once caching is validated, add LRU eviction to prevent disk bloat
- [ ] **Background mode/autostart** — Once manual refresh works, add timer for automatic updates (user-requested)
- [ ] **Additional sources** — After Bing+Spotlight stability proven, add NASA APOD, Unsplash, Wallhaven
- [ ] **Open wallpaper URL** — Click to view source (attributions already collected from APIs)
- [ ] **Run in background toggle** — Allow app to continue without open window (for autostart)

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Multi-monitor support** — Low early adoption usage; HydraPaper exists for this niche
- [ ] **Iced UI toolkit migration** — COSMIC native experience; requires clean engine architecture (do this first!)
- [ ] **Advanced search/filtering** — WonderWall does this well; different app category (gallery vs automatic)
- [ ] **User favorites/bookmarks** — Nice to have, but not core "automatic wallpaper" value prop
- [ ] **Slideshow source** — Local folder rotation; Damask has this, but online sources are differentiator
- [ ] **Extensive source catalog** — Add proven sources only; avoid unmaintained APIs

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Manual refresh | HIGH | LOW | P1 |
| Bing Wallpaper source | HIGH | LOW | P1 |
| Spotlight source | HIGH | MEDIUM | P1 |
| Wallpaper preview | HIGH | MEDIUM | P1 |
| GNOME wallpaper setting | HIGH | MEDIUM | P1 |
| COSMIC wallpaper setting | HIGH | MEDIUM | P1 |
| Error handling with toasts | HIGH | MEDIUM | P1 |
| Settings window | MEDIUM | LOW | P1 |
| Basic caching | MEDIUM | MEDIUM | P1 |
| Cache cleanup | MEDIUM | MEDIUM | P2 |
| Background mode/autostart | MEDIUM | HIGH | P2 |
| Additional sources (APOD, Unsplash) | MEDIUM | LOW | P2 |
| Open wallpaper URL | LOW | LOW | P2 |
| Run in background toggle | LOW | LOW | P2 |
| Multi-monitor support | LOW | HIGH | P3 |
| Iced UI migration | MEDIUM | HIGH | P3 |
| Advanced search/filtering | LOW | HIGH | P3 |
| User favorites/bookmarks | LOW | MEDIUM | P3 |
| Slideshow source | LOW | LOW | P3 |
| Extensive source catalog | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch (v1 MVP)
- P2: Should have, add when possible (v1.x)
- P3: Nice to have, future consideration (v2+)

## Competitor Feature Analysis

| Feature | Damask (Vala) | Picture of the Day | WonderWall | Our Approach |
|---------|----------------|-------------------|------------|--------------|
| **Sources** | 8+ (Bing, Spotlight, APOD, EarthView, Wallhaven, Unsplash, local, slideshow) | 5 (NASA APOD, Bing, Simon Stålenhag, Wikimedia POD, NASA Earth Observatory) | 7+ (Wallhaven, DeviantArt, Bing, Unsplash, etc.) | Start with 2 (Bing, Spotlight); add proven sources in v1.x |
| **Refresh** | Manual + automatic (timer) | Automatic (daily) | Manual (browse) | Manual only for MVP; add timer in v1.x |
| **Preview** | Yes (thumbnail, title, description, attribution) | Yes (preview images, pick favorite) | Yes (preview, zoom, fullscreen) | Yes (thumbnail, title, description, attribution) |
| **Multi-monitor** | No (same wallpaper on all monitors) | No | No | No for MVP; defer to v2 |
| **Multi-DE support** | GNOME (GSettings) | GNOME (GSettings) | Cross-platform | GNOME + COSMIC (runtime detection) |
| **Caching** | Yes (indefinite) | Yes | Yes (offline library) | Yes (with LRU eviction - differentiator) |
| **Error handling** | Toasts | Not specified | Not specified | Toasts (all errors visible) |
| **Language** | Vala | Rust (Python backend?) | Electron + Vue | Rust (native GTK) |
| **Background mode** | Yes (autostart, run without window) | Yes (automatic updates) | No | No for MVP; add in v1.x |
| **License** | GPL-3.0 | GPL-3.0 | Proprietary | GPL-3.0 (same as Damask) |

**Key differentiators:**
1. **Rust native implementation** — Better than Vala (Damask) for performance/maintenance
2. **Clean engine/UI architecture** — Enables future Iced migration (COSMIC native)
3. **Proper cache cleanup** — Competitors cache indefinitely; our LRU prevents bloat
4. **COSMIC support** — Newer DE, underserved market opportunity
5. **Async non-blocking** — Modern UX vs potentially blocking Vala UI

## Sources

- [Damask source code](https://gitlab.gnome.org/subpop/damask) — HIGH confidence (reference implementation, analyzed directly)
- [Flathub Damask page](https://flathub.org/apps/details/app.drey.Damask) — MEDIUM confidence (feature list, user-facing description)
- [Flathub wallpaper apps collection](https://flathub.org/apps/collection/tag/wallpaper) — MEDIUM confidence (competitor landscape)
- [Flathub HydraPaper page](https://flathub.org/apps/details/org.gabmus.hydrapaper) — MEDIUM confidence (multi-monitor niche)
- [Flathub WonderWall page](https://flathub.org/apps/details/com.ktechpit.wonderwall) — MEDIUM confidence (gallery/browsing features)
- [Flathub Picture of the Day page](https://flathub.org/apps/details/de.swsnr.pictureoftheday) — MEDIUM confidence (multiple sources)
- [Flathub Wallpaper Downloader page](https://flathub.org/apps/details/es.estoes.wallpaperDownloader) — MEDIUM confidence (automatic downloads)
- [PROJECT.md](/home/widi/GitHub/damask-rs/.planning/PROJECT.md) — HIGH confidence (project constraints, MVP scope)
- [PITFALLS.md](/home/widi/GitHub/damask-rs/.planning/research/PITFALLS.md) — HIGH confidence (domain-specific risks)
- [STACK.md](/home/widi/GitHub/damask-rs/.planning/research/STACK.md) — HIGH confidence (technology stack)

---
*Feature research for: Linux wallpaper application (automatic desktop wallpaper from online sources)*
*Researched: 2026-04-13*
