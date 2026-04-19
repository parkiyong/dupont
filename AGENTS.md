<!-- GSD:project-start source:PROJECT.md -->
## Project

**Dupont**

Dupont is a Rust desktop wallpaper application for Linux desktop environments. It automatically fetches and sets desktop wallpapers from various online sources. The initial release targets GNOME and COSMIC desktop environments, starting with Microsoft Bing Wallpaper of the Day and Microsoft Spotlight as the supported wallpaper sources.

**Core Value:** Users can automatically set their desktop wallpaper from online sources (Bing, Spotlight) with a simple, native Linux application.

### Constraints

- **UI Toolkit**: GTK-rs for v1 — Must be easily swappable to Iced or other toolkits later via clean architecture
- **Desktop Environments**: GNOME and COSMIC only — Must handle wallpaper setting APIs for both DEs
- **Timeline**: Weeks — MVP must be working quickly, prioritize completion over features
- **Sources**: Only Bing and Spotlight for v1
- **Testing**: No test coverage in v1 — Skip unit tests to focus on getting software working
- **Learning Focus**: Beginner-friendly code structure — Avoid over-optimizations, prioritize clarity
- **Scope**: Minimal MVP — Basic fetch and set wallpaper functionality with simple UI, no advanced features
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.93+ | Core language | Minimum version required by relm4 0.10+, provides safety and performance |
| gtk4 | 0.11 | GUI framework (bindings) | Mature, native GNOME experience, excellent documentation, battle-tested in production apps (Fractal, Amberol, 56+ apps on gtk-rs.org) |
| relm4 | 0.10 | UI framework (Elm-inspired) | Idiomatic Rust GUI development, built on gtk4-rs, simplifies state management, excellent async support with ComponentSender, accelerates development for beginners |
| gio | 0.20 | Desktop integration (gsettings) | Part of gtk-rs ecosystem, provides Settings API for wallpaper setting on GNOME/COSMIC, cross-platform compatibility |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| reqwest | 0.12 | HTTP client (async) | Fetching wallpapers from Bing API, use with `json` feature for JSON parsing |
| serde | 1.0 | Serialization framework | Required by reqwest for JSON parsing of API responses |
| serde_json | 1.0 | JSON serialization/deserialization | Parsing Bing Wallpaper API responses |
| tokio | 1.51 | Async runtime | Required by reqwest for async HTTP operations, use with `rt-multi-thread` feature |
| dirs | 5.0 | XDG directory paths | Getting cache, config, and data directories following Linux standards (~/.cache, ~/.config, ~/.local/share) |
| thiserror | 2.0 | Error handling (domain-specific) | Define custom error types for core engine (fetch errors, cache errors, wallpaper setting errors) |
| anyhow | 1.0 | Error handling (application) | Easy error propagation in UI layer, convert domain errors to application errors |
| log | 0.4 | Logging facade | Structured logging API |
| env_logger | 0.11 | Logger implementation | Runtime-configurable logger for development |
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| cargo | Build tool (standard) | Rust's package manager and build system |
| rustup | Version manager | Manage Rust toolchain versions |
| clippy | Linting | Rust component, enables additional lints: `rustup component add clippy` |
| rustfmt | Code formatting | Rust component, auto-format code: `rustup component add rustfmt` |
| gtk4-update-icon-cache | Icon cache management | Part of GTK4 installation, update application icons |
| glib-compile-resources | Resource compilation | Part of GLib installation, compile GResource files (icons, UI definitions) |
| desktop-file-validate | Desktop file validation | Part of desktop-file-utils, validate .desktop files for Linux integration |
## Installation
# System dependencies (Linux - Debian/Ubuntu)
# System dependencies (Linux - Fedora)
# System dependencies (Linux - Arch)
# Core dependencies
# Feature flags for core dependencies
# Optional: image processing (for resizing/scaling wallpapers)
# Dev dependencies
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| gtk4-rs + relm4 | gtk4-rs (direct) | When you need fine-grained control over GTK4 internals or have unusual async patterns; use relm4 for faster, more maintainable code in most cases |
| gtk4-rs + relm4 | iced | For COSMIC-native UI development or pure Rust cross-platform apps; less mature than GTK4, smaller ecosystem, different async model |
| reqwest | ureq | For simple blocking HTTP requests only; reqwest provides async support, connection pooling, and is more feature-complete |
| tokio | async-std | If you prefer async-std's API style; tokio has larger ecosystem, better async TLS support, and is the de facto standard |
| log + env_logger | tracing + tracing-subscriber | For production applications requiring structured logging, async-aware tracing, or complex log filtering; log + env_logger is simpler for MVP |
| thiserror + anyhow | only anyhow | For small apps without domain-specific error needs; thiserror provides better error messages and type safety for core engine errors |
| dirs | xdg | Never use xdg crate (deprecated, unmaintained since 2020); dirs is actively maintained and provides cleaner API |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| gtk-rs (GTK3) | GTK3 is deprecated, GTK4 is current stable, better performance, native Wayland support, future-proofing | gtk4-rs (GTK4) |
| xdg crate | Deprecated since 2020, unmaintained, last update 5+ years ago, may have security issues | dirs crate (actively maintained, v5.0+) |
| relm (old version) | Original relm project, not compatible with GTK4, lacks modern Elm-inspired architecture | relm4 (complete rewrite, GTK4-compatible, active development) |
| curl-rust | More complex API, less idiomatic Rust, smaller community than reqwest | reqwest (standard HTTP client in Rust ecosystem) |
| synchronous blocking HTTP | Blocks GTK4 main loop, freezes UI during network requests, poor user experience | reqwest async with tokio runtime |
| manual path handling for config/cache | Non-standard locations, violates XDG Base Directory Specification, harder for users to find files | dirs crate (follows XDG standards: ~/.cache, ~/.config, ~/.local/share) |
| unwrap() everywhere | Panics crash the application, poor error handling, bad UX | thiserror (domain errors) + anyhow (app errors) with proper error propagation |
| Direct shell command execution (calling `gsettings` command) | Fragile, platform-dependent, error handling via exit codes, not type-safe | gio::Settings API (type-safe, integrates with GLib main loop) |
## Stack Patterns by Variant
- Use gtk4 + relm4 with libadwaita (optional, for GNOME HIG compliance)
- Because: libadwaita provides GNOME design language components, better integration, modern look-and-feel
- Use gtk4 + relm4 for v1, but architect core engine with trait-based abstraction
- Because: Clean separation enables UI toolkit swap without rewriting core logic (fetch, cache, wallpaper setting)
- Pattern: Define `WallpaperEngine` trait, implement in core crate, UI layer uses trait methods
- Use tokio with tokio::time::interval for periodic tasks
- Because: Tokio integrates well with relm4's async model, ComponentSender can send messages to UI on timer events
- Use image crate 0.25 for format decoding/encoding
- Because: Most comprehensive image format support in Rust, actively maintained, good performance
- Migrate from log + env_logger to tracing + tracing-subscriber + tracing-appender
- Because: Structured fields, async-aware, better filtering, file logging with rotation
## Version Compatibility
| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| relm4 0.10 | gtk4 0.11.x, tokio 1.51+ | relm4 depends on gtk4 0.11.2 and tokio 1.51 internally |
| reqwest 0.12 | tokio 1.x | reqwest 0.12 uses tokio 1.x for async runtime |
| gio 0.20 | glib 0.20 | Part of gtk-rs-core, version-synced with glib |
| gtk4 0.11 | GTK 4.14+ (system library) | Must have GTK4 development libraries installed on system |
| dirs 5.0 | No runtime dependencies | Pure Rust, cross-platform |
| thiserror 2.0 | serde 1.x | Optional derive for Serialize/Deserialize on Error types |
## Architecture Pattern for Clean UI Swapping
## Sources
- gtk-rs.org — GTK4 installation, available crates, ecosystem overview, app examples (HIGH confidence)
- relm4.org — Relm4 documentation, async patterns, ComponentSender API, tokio integration (HIGH confidence)
- docs.rs/crates/reqwest — Reqwest API documentation, features, async patterns (HIGH confidence)
- crates.io — Current stable versions: gtk4 0.11, relm4 0.10, reqwest 0.12, tokio 1.51, dirs 5.0 (HIGH confidence)
- GNOME developer documentation — GSettings API for wallpaper setting: `org.gnome.desktop.background` schema, `picture-uri` key (HIGH confidence)
- System76 COSMIC documentation — GNOME compatibility mode, wallpaper setting via gsettings (MEDIUM confidence — COSMIC-specific APIs less documented, gsettings approach verified to work)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
