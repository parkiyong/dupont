# Domain Pitfalls

**Domain:** Rust desktop wallpaper application (GNOME/COSMIC)
**Researched:** 2026-04-13
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: Blocking GTK Main Thread with I/O Operations

**What goes wrong:**
Application becomes unresponsive during HTTP requests, image decoding, or file operations. Users see frozen UI and force-quit the application, leading to corrupted caches and incomplete operations.

**Why it happens:**
Developers new to GTK-rs often perform blocking operations (HTTP requests, image decoding, file I/O) directly in signal handlers or UI callbacks. GTK requires all UI updates on the main thread, but developers forget that I/O operations also block the main loop, preventing event processing.

**How to avoid:**
- Use async HTTP clients (reqwest::Client instead of reqwest::blocking)
- Spawn heavy image processing in separate threads with `glib::spawn_future_local` or Rayon
- Use `glib::idle_add` or `glib::timeout_add` to schedule UI updates after background work
- Never call `.await` directly in signal handlers without spawning a future

**Warning signs:**
- Application UI freezes during "Refresh" operation
- Mouse events (clicks, hovers) stop responding
- Application windows don't repaint during wallpaper fetch
- Users report "app hung" or "force quit"

**Phase to address:**
Phase 1 - Core Engine Setup. Establish async patterns before implementing wallpaper fetching.

---

### Pitfall 2: Cross-Desktop Environment Compatibility Assumptions

**What goes wrong:**
Application works on GNOME but silently fails on COSMIC, or vice versa. Wallpaper appears to set but doesn't update, or errors are silently swallowed due to incorrect API assumptions.

**Why it happens:**
GNOME uses GSettings (via gsettings command-line tool or gio::Settings) with specific schema paths like `org.gnome.desktop.background`. COSMIC is newer and uses different mechanisms (cosmic-settings, potentially different schemas). Developers often hardcode GNOME-specific paths without runtime detection.

**How to avoid:**
- Detect desktop environment at runtime via environment variables (`XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`)
- Abstract wallpaper setting behind a trait: `trait WallpaperSetter { fn set(&self, path: &Path) -> Result<()> }`
- Implement separate strategies for GNOME and COSMIC
- Test on both DEs before considering features "complete"
- Provide clear error messages when wallpaper setting fails (e.g., "Failed to set wallpaper on COSMIC: ...")

**Warning signs:**
- Works in one DE but not the other
- Error logs show "file not found" for expected wallpaper paths
- Setting appears to succeed in UI but wallpaper doesn't change
- Silent failures in wallpaper setting operation

**Phase to address:**
Phase 2 - Wallpaper Setting Integration. Implement abstraction layer before adding any DE-specific code.

---

### Pitfall 3: Image Caching Without Cleanup Strategy

**What goes wrong:**
Application disk usage grows indefinitely as wallpapers are cached but never removed. After months of use, cache consumes gigabytes of storage. Users uninstall due to "bloatware" perception.

**Why it happens:**
Developers implement caching for performance (avoid re-downloading) but don't design a cleanup strategy. LRU (Least Recently Used) or size-based limits are omitted in favor of "implement caching first, cleanup later."

**How to avoid:**
- Design cache schema before implementing: max size (e.g., 500MB), max age (e.g., 30 days), max count (e.g., 50 wallpapers)
- Implement LRU eviction: track access timestamps, remove oldest entries when limits exceeded
- Use a dedicated cache directory with clear structure: `~/.cache/damask-rs/{source-id}/{image-hash}.{ext}`
- Expose cache statistics in UI (size, count) for transparency
- Add "Clear Cache" button in settings (even for MVP)

**Warning signs:**
- Cache directory size grows with each refresh
- No mechanism to remove old wallpapers
- Users report "app is taking too much space"
- No cache management in UI

**Phase to address:**
Phase 1 - Core Engine Setup. Design cache schema and cleanup before implementing caching logic.

---

### Pitfall 4: GSettings Schema Path Assumptions

**What goes wrong:**
Application fails to set wallpaper because it assumes the wrong GSettings schema path. GNOME schemas can vary by version (e.g., `org.gnome.desktop.background` vs `org.gnome.desktop.background.picture-options`), and developers hardcode paths without verification.

**Why it happens:**
Developers copy code from tutorials or older GNOME applications without considering schema changes across GNOME versions. COSMIC may have entirely different schema names. Schema paths are string literals, so no compile-time verification.

**How to avoid:**
- Verify schema exists at runtime using `gio::SettingsSchemaSource::default().lookup(schema_name)`
- Fall back to alternative schema names or error with clear message
- Use `gsettings list-schemas` and `gsettings list-keys` during development to verify paths
- Document which GNOME versions are tested (e.g., "GNOME 45+")
- Provide schema path as configuration option (even if hardcoded for MVP)

**Warning signs:**
- GSettings errors in logs: "Schema X not found"
- Wallpapers don't set despite no visible error
- Different behavior on different GNOME versions
- COSMIC completely breaks with schema errors

**Phase to address:**
Phase 2 - Wallpaper Setting Integration. Add schema verification before first wallpaper set operation.

---

### Pitfall 5: Image Format Assumptions Without Robust Decoding

**What goes wrong:**
Application crashes when encountering unexpected image formats or malformed files. Bing/Spotlight APIs may return formats not handled by naive decoders (e.g., WebP, HEIC, large JPEGs with corrupted headers).

**Why it happens:**
Developers use simple `image::open()` which returns errors, but assume all valid responses from APIs are decodable. Error handling treats `ImageError` as "download failed" rather than "unsupported format."

**How to avoid:**
- Use `image::ImageReader::open()` with format guessing enabled
- Handle all `image::ImageError` variants explicitly: `UnsupportedFormat`, `IoError`, `ParameterError`
- Pre-verify images before caching: decode in memory, re-encode to standard format (e.g., PNG/JPEG)
- Set image library limits to prevent OOM: `image::io::Limits` (max width, max size)
- Log image metadata on decode failure for debugging

**Warning signs:**
- Application crashes on specific wallpaper URLs
- Image errors in logs with "unsupported format"
- Some wallpapers work, others don't
- No error message displayed to user when decode fails

**Phase to address:**
Phase 1 - Core Engine Setup. Implement robust image decoding with error handling before caching logic.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|------------|-------------------|------------------|------------------|
| Hardcode GNOME/COSMIC wallpaper paths | Get MVP working quickly | Breaks on DE updates, hard to test on multiple systems | Never - abstraction is cheap |
| Ignore image format edge cases | Handle 99% of wallpapers successfully | Crashes on 1%, poor user experience, tech debt accumulation | Only for prototypes, never for MVP |
| Skip cache cleanup for "later" | Faster initial development, simpler code | Users uninstall due to bloat, negative reviews, difficult to add retroactively | Never - design from start |
| Use blocking HTTP calls in async context | Simpler code, no async complexity | UI freezes, poor performance, hard to fix later | Never - use async from day 1 |
| Monolithic error handling | Easier to implement initially | Impossible to debug, poor UX, tech debt | Only for quick prototypes |
| Assume single monitor | Simpler code path | Multi-monitor users disappointed, half-baked experience | Acceptable for MVP if documented, but flag for v2 |
| Skip DE detection | Faster to implement | Breaks on unexpected DEs, hard to extend | Never - cheap to add runtime check |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|------------------|------------------|
| **Bing Wallpaper API** | Assuming all responses are JPEG; ignoring rate limits; not handling pagination | Verify format from response headers; implement backoff on rate limiting; use async client with timeout |
| **Microsoft Spotlight** | Not parsing complex JSON correctly; assuming image URLs are always valid | Use serde with strict schema validation; validate URLs before download; handle missing fields gracefully |
| **GSettings (GNOME)** | Assuming schema paths are static; not handling "changed" signals; missing schema verification | Verify schema at runtime; listen to `changed` signals for live wallpaper updates; provide fallback paths |
| **COSMIC Settings** | Assuming it uses same mechanisms as GNOME; not finding documentation | Research cosmic-settings D-Bus APIs; implement fallback to manual file writing if API unavailable |
| **reqwest HTTP** | Using blocking client in async app; not setting timeouts; ignoring TLS errors | Use `reqwest::Client` (async); set timeout to 10-30s; handle TLS errors with clear messages |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|------------------|
| **Loading full wallpapers into memory** | High RAM usage, UI freezes on large images | Stream decode with `image::ImageReader`; use thumbnails for UI previews, full images only for setting | At 10+ cached wallpapers, especially 4K+ resolution |
| **Sequential HTTP requests** | Slow refresh times (wallpaper by wallpaper) | Use `futures::join_all` for parallel downloads; limit concurrency to 4-8 requests | When fetching from multiple sources simultaneously |
| **Cache directory traversal on startup** | Slow app launch (seconds to list cached wallpapers) | Maintain index file (`~/.cache/damask-rs/index.json`); load metadata without file I/O | At 50+ cached wallpapers |
| **Inefficient image format conversion** | CPU spikes during wallpaper refresh | Decode once, cache both original and converted formats; reuse decoder instances | When processing >5 wallpapers per session |
| **UI thread blocking on file operations** | Stuttering during wallpaper list scroll | Use thread pools for file I/O; implement virtualized list widgets | At 20+ wallpapers in UI list |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|----------|------|------------|
| **Trusting API URLs without validation** | Open redirect attacks, downloading malicious content | Validate URLs are from expected domains (e.g., `*.bing.com`); check for URL encoding tricks |
| **Not sanitizing filenames** | Path traversal, overwriting system files | Use `path::PathBuf::push` with strict validation; restrict to cache directory; reject paths with `..` or absolute paths |
| **Ignoring TLS certificate errors** | Man-in-the-middle attacks | Use `reqwest` default TLS; never disable verification; provide clear error on cert failure |
| **Exposing cache directory in Flatpak** | Data leakage, unexpected sandbox escapes | Don't request broader Flatpak permissions than needed; document cache location for users; use `--filesystem=xdg-cache/damask-rs` instead of `home` |
| **Logging sensitive URLs** | Information leakage in logs | Redact API keys or tokens from logs; don't log full URLs with query parameters |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|----------|-------------|-----------------|
| **No progress indication during refresh** | Users think app is broken, force-quit, corrupt cache | Show progress bar (downloading, decoding, setting); cancelable operation; time estimate |
| **Silent failures** | Users don't know what went wrong, can't report bugs | Show error dialogs with actionable messages (e.g., "Failed to set wallpaper: Check desktop environment") |
| **Blocking UI during long operations** | Application appears frozen, users kill it | Always spawn async work; show spinner or skeleton UI; allow cancellation |
| **No preview before setting** | Users download wallpapers they don't want | Show thumbnail preview in list; right-click to "Set as wallpaper" vs auto-set on click |
| **Inconsistent error handling** | Some errors show dialogs, others log silently, users confused | Centralize error handling function; always show user-facing errors; log technical details separately |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Wallpaper fetching:** Often missing [retry logic] — verify [exponential backoff on network errors, 3-5 retry attempts]
- [ ] **Image caching:** Often missing [eviction strategy] — verify [cache size limits, age-based cleanup, LRU tracking]
- [ ] **Wallpaper setting:** Often missing [fallback mechanisms] — verify [schema path fallbacks, DE detection, clear error messages]
- [ ] **Error handling:** Often missing [user-friendly messages] — verify [all errors show in UI, technical details in logs]
- [ ] **Cross-DE support:** Often missing [runtime detection] — verify [detects GNOME vs COSMIC, tests on both, errors on unsupported DEs]
- [ ] **Image formats:** Often missing [format conversion] — verify [handles WebP, HEIC, corrupted images, converts to standard format]

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|----------|---------------|----------------|
| **Main thread blocking** | LOW | Refactor I/O to async; add loading indicators; test with slow network simulated (e.g., 100ms delay) |
| **Wrong DE assumptions** | MEDIUM | Add runtime DE detection; implement trait abstraction; add integration tests for each DE; release patch |
| **Cache bloat** | MEDIUM | Implement cleanup cron job; add LRU eviction; expose cache management in UI; ship migration script to clean old cache |
| **GSettings schema errors** | LOW | Add schema verification on startup; provide fallback to direct file writing; document supported GNOME versions |
| **Image decode crashes** | LOW | Add format detection loop; wrap decode in panic-safe code; log image metadata on failure; add format blacklist |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|----------|------------------|--------------|
| Main thread blocking | Phase 1 - Core Engine Setup | Add slow network simulation test; verify UI remains responsive during refresh |
| Cross-DE compatibility | Phase 2 - Wallpaper Setting Integration | Test on both GNOME and COSMIC; verify DE detection works |
| Image caching without cleanup | Phase 1 - Core Engine Setup | Add cache size test: after 50 wallpapers, verify size under limit |
| GSettings schema assumptions | Phase 2 - Wallpaper Setting Integration | Test on multiple GNOME versions; verify fallback paths work |
| Image format assumptions | Phase 1 - Core Engine Setup | Test with WebP, HEIC, corrupted images; verify graceful errors |
| Silent failures | Phase 2 - Wallpaper Setting Integration | Trigger all error paths; verify each shows user-facing message |
| No progress indication | Phase 3 - UI Implementation | Start refresh operation; verify progress bar appears and updates |
| File path traversal | Phase 1 - Core Engine Setup | Add path fuzzing tests; verify paths are sanitized |
| TLS/HTTPS errors | Phase 1 - Core Engine Setup | Use expired cert test server; verify error shown, not ignored |

## Sources

- [gtk-rs official documentation](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4) - MEDIUM confidence (official docs)
- [GSettings API documentation](https://developer.gnome.org/gio/stable/gio-GSettings.html) - MEDIUM confidence (official docs)
- [reqwest HTTP client docs](https://docs.rs/reqwest/latest/reqwest/) - HIGH confidence (official docs)
- [image crate documentation](https://docs.rs/image/latest/image/) - HIGH confidence (official docs)
- [serde serialization framework](https://docs.rs/serde/latest/serde/) - HIGH confidence (official docs)
- [Flatpak common issues](https://github.com/flatpak/flatpak/issues) - MEDIUM confidence (issue tracker)
- [COSMIC panel issues](https://github.com/pop-os/cosmic-panel/issues) - LOW confidence (limited COSMIC docs available)
- [Flathub app requirements](https://github.com/flathub/flathub/wiki) - MEDIUM confidence (community guidelines)

---
*Pitfalls research for: Rust desktop wallpaper application (GNOME/COSMIC)*
*Researched: 2026-04-13*
