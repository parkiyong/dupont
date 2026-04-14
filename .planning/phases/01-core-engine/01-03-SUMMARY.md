---
phase: 01-core-engine
plan: 03
subsystem: core-engine
tags: [rust, image-validation, lru-cache, tokio-fs, xdg-cache, eviction, oom-protection]

# Dependency graph
requires:
  - phase: 01-core-engine/01
    provides: "Wallpaper struct, CacheError enum, tokio/image/dirs workspace deps"
provides:
  - "Cache manager with LRU eviction (500MB, 50 images, 30 days)"
  - "Image format validation with OOM protection (8192x8192, 50MB)"
  - "CacheConfig for customizable cache limits"
  - "CacheStats for cache monitoring"
  - "tokio 'fs' feature enabled in workspace"
affects: [02-desktop-integration, 03-user-interface, 04-integration-testing]

# Tech tracking
tech-stack:
  added: []
  patterns: [lru-eviction, image-validation-with-limits, xdg-cache-directory, format-guessing-validation]

key-files:
  created:
    - domain/src/cache.rs
  modified:
    - domain/src/lib.rs
    - Cargo.toml

key-decisions:
  - "Used image::ImageReader API directly (not deprecated image::io::Reader) for format guessing with limits"
  - "Combined validate_and_detect into single method to avoid double image decode"
  - "Added tokio 'fs' feature to workspace for async file I/O in cache"

patterns-established:
  - "LRU eviction: age-based first (30 days), then least-recently-accessed until within limits"
  - "Image validation: decode with Limits (max 8192x8192, 50MB alloc) to prevent OOM from malicious/corrupted images"
  - "Cache entry tracking: in-memory Vec<CacheEntry> with file system as source of truth"

requirements-completed: [CORE-05, CORE-06]

# Metrics
duration: 2min
completed: 2026-04-14
---

# Phase 1 Plan 03: Image Cache with LRU Eviction Summary

**Cache manager with LRU eviction (500MB/50 images/30 days), image format validation (JPEG/PNG/WebP), and OOM protection via image::Limits**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-14T00:34:20Z
- **Completed:** 2026-04-14T00:37:02Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Cache manager storing wallpapers in `~/.cache/damask/` with configurable limits
- LRU eviction: removes entries older than 30 days first, then least-recently-accessed until within size/count limits
- Image format validation using `image::ImageReader` with OOM limits (8192x8192 max dimensions, 50MB max alloc)
- Format detection via `with_guessed_format()` — supports JPEG, PNG, WebP; rejects HEIC with clear error
- Async file operations via `tokio::fs` for non-blocking cache reads/writes

## Task Commits

Each task was committed atomically:

1. **Task 1: Define cache configuration and structure** - `a82b707` (feat)
2. **Task 1 fix: Resolve image 0.25 API and tokio fs issues** - `bc7464d` (fix)
3. **Task 2: Update domain lib.rs to export cache** - `5698f3c` (feat)

## Files Created/Modified
- `domain/src/cache.rs` — Cache manager (408 lines): Cache, CacheConfig, CacheStats, CacheEntry, validation, LRU eviction
- `domain/src/lib.rs` — Added `pub mod cache` and re-exported Cache, CacheConfig, CacheStats
- `Cargo.toml` — Added `fs` feature to tokio workspace dependency

## Decisions Made
- Used `image::ImageReader` (current API) instead of deprecated `image::io::Reader`
- Set image limits via `Limits` struct mutation (struct is `#[non_exhaustive]`, can't use literal) and `reader.limits()` method
- Combined `validate_image` and `detect_extension` into single `validate_and_detect` to avoid decoding image twice
- Captured `reader.format()` before `reader.decode()` since decode consumes self
- Added `to_unix_secs` helper to convert `SystemTime` to Unix timestamps with proper error handling

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added tokio `fs` feature to workspace Cargo.toml**
- **Found during:** Task 1 (cargo check)
- **Issue:** `tokio::fs::read_dir`, `tokio::fs::metadata`, `tokio::fs::write`, `tokio::fs::remove_file` all require the `fs` feature, which wasn't enabled in workspace tokio dependency
- **Fix:** Added `"fs"` to tokio features in root Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** `cargo check --package damask-domain` passes
- **Committed in:** `bc7464d`

**2. [Rule 1 - Bug] Fixed image 0.25 API usage**
- **Found during:** Task 1 (cargo check)
- **Issue:** Plan template used `image::io::Reader` (deprecated), `image::io::Limits` builder methods (fields are public, not methods), and `reader.set_limits()` (method is `reader.limits()`). Also `Limits` is `#[non_exhaustive]` preventing struct literal construction.
- **Fix:** Used `image::ImageReader`, created `Limits::default()` then mutated public fields, used `reader.limits(limits)` method
- **Files modified:** domain/src/cache.rs
- **Committed in:** `bc7464d`

**3. [Rule 1 - Bug] Fixed borrow after move in validate_and_detect**
- **Found during:** Task 1 (cargo check)
- **Issue:** `reader.decode()` takes `self` by value (moves), so `reader.format()` called after decode is a use-after-move
- **Fix:** Captured `reader.format()` into a local variable before calling `decode()`
- **Files modified:** domain/src/cache.rs
- **Committed in:** `bc7464d`

**4. [Rule 1 - Bug] Fixed borrow checker conflicts in get_or_download and evict_lru**
- **Found during:** Task 1 (cargo check)
- **Issue:** `find_entry()` borrows `self` immutably, but `update_access_time()` borrows `self` mutably — can't hold both. Same pattern in `evict_lru_until_within_limits` with `min_by_key` and `remove_entry`.
- **Fix:** Clone the path/ID before the mutable borrow in both locations
- **Files modified:** domain/src/cache.rs
- **Committed in:** `bc7464d`

**5. [Rule 1 - Bug] Fixed SystemTimeError to io::Error conversion**
- **Found during:** Task 1 (cargo check)
- **Issue:** `SystemTime::duration_since()` returns `Result<Duration, SystemTimeError>`, not `io::Error`. `CacheError::IoError` wraps `std::io::Error` via `#[from]`, so direct `map_err(CacheError::IoError)` doesn't work.
- **Fix:** Used `map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))` in `to_unix_secs` helper
- **Files modified:** domain/src/cache.rs
- **Committed in:** `bc7464d`

**6. [Rule 1 - Bug] Removed broken `impl Default for Cache`**
- **Found during:** Task 1 (pre-commit review)
- **Issue:** Plan template had `impl Default for Cache { fn default() -> Result<Self, CacheError> }` which violates the `Default` trait signature (must return `Self`, not `Result<Self, E>`)
- **Fix:** Removed the impl — `CacheConfig` already has `Default`, and `Cache::with_defaults()` provides the factory method
- **Files modified:** domain/src/cache.rs
- **Committed in:** `a82b707`

---

**Total deviations:** 6 auto-fixed (5 bugs, 1 blocking)
**Impact on plan:** All fixes were necessary for compilation and correctness. Plan's code template was written against an older/different image crate API. No scope creep — all changes within Task 1 scope.

## Issues Encountered
- Plan's code template used outdated image crate API (pre-0.25 style) — required updating to current `ImageReader`, `Limits`, and method signatures
- image 0.25 has `#[non_exhaustive]` on `Limits` struct, preventing struct literal construction from external crates

## Threat Model Compliance

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-01-07: Cache file path spoofing | Uses `wallpaper.id` (from API) as filename | ✅ Mitigated |
| T-01-08: Image format validation | Decodes with ImageReader to verify validity | ✅ Mitigated |
| T-01-09: DoS via large/corrupted images | Limits: max 8192x8192, max 50MB alloc | ✅ Mitigated |
| T-01-10: Disk space exhaustion | 500MB/50 images/30 days with LRU eviction | ✅ Mitigated |
| T-01-11: Cache directory disclosure | XDG-compliant `~/.cache/damask/` | ✅ Mitigated |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Cache manager ready for integration with UI layer (Phase 3) — `get_or_download()` provides simple async API
- BingSource and SpotlightSource from Plan 02 can feed wallpapers directly into Cache
- Image validation ensures only valid JPEG/PNG/WebP files enter cache
- Phase 2 desktop integration can use cached file paths for wallpaper setting

## Self-Check: PASSED

All 1 created file exists. All 3 task commits verified in git history. `cargo check --workspace` passes with zero errors.

---
*Phase: 01-core-engine*
*Completed: 2026-04-14*
