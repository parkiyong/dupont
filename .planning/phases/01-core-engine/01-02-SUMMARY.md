---
phase: 01-core-engine
plan: 02
subsystem: core-engine
tags: [rust, reqwest, serde, async, bing-api, spotlight-api, http-client, json-parsing]

# Dependency graph
requires:
  - phase: 01-core-engine/01
    provides: "Source trait, Wallpaper struct, SourceError enum, async-trait, reqwest workspace dep"
provides:
  - "BingSource implementation with async HTTP fetching from Bing Wallpaper of the Day API"
  - "SpotlightSource implementation with async HTTP fetching from Microsoft Spotlight API"
  - "Exponential backoff retry pattern for HTTP 429 rate limiting"
  - "Sources module with re-exports for BingSource and SpotlightSource"
affects: [02-desktop-integration, 03-user-interface, 04-integration-testing]

# Tech tracking
tech-stack:
  added: []
  patterns: [exponential-backoff-retry, serde-nested-json, builder-pattern-source-config]

key-files:
  created:
    - domain/src/sources/mod.rs
    - domain/src/sources/bing.rs
    - domain/src/sources/spotlight.rs
  modified:
    - domain/src/lib.rs

key-decisions:
  - "Used Wallpaper::new() constructor instead of struct literal for consistency with builder pattern"
  - "Suppressed dead_code warnings on serde-only fields (attribution_url) with #[allow(dead_code)]"

patterns-established:
  - "Retry pattern: exponential backoff (2s, 4s, 8s) on HTTP 429, max 3 retries, then return RateLimited error"
  - "Source configuration: new() for defaults, with_market()/with_locale() for customization, with_resolution() builder method"
  - "Attribution extraction: parse (© Holder) suffix from copyright/description strings with fallback to source name"

requirements-completed: [CORE-01, CORE-02, CORE-03]

# Metrics
duration: 1min
completed: 2026-04-14
---

# Phase 1 Plan 02: Bing and Spotlight Source Implementations Summary

**BingSource and SpotlightSource with async reqwest HTTP fetching, serde JSON parsing, exponential backoff retry on rate limiting, and 10-second timeout**

## Performance

- **Duration:** 1 min
- **Started:** 2026-04-14T00:32:54Z
- **Completed:** 2026-04-14T00:33:47Z
- **Tasks:** 4
- **Files modified:** 4

## Accomplishments
- BingSource fetching wallpapers from Bing Wallpaper of the Day API with configurable market locale
- SpotlightSource fetching wallpapers from Microsoft Spotlight API with configurable locale and resolution
- Both sources implement Source trait with async fetch, proper error handling, and rate limit retry
- Domain crate exports both source implementations for consumption by UI layer

## Task Commits

Each task was committed atomically:

1. **Task 1: Create source module structure** - `ef0560c` (feat)
2. **Task 2: Implement BingSource with async fetching** - `ca10c6d` (feat)
3. **Task 3: Implement SpotlightSource with async fetching** - `85e99b8` (feat)
4. **Task 4: Update domain lib.rs to export sources** - `7f8d70e` (feat)

## Files Created/Modified
- `domain/src/sources/mod.rs` - Module structure with public exports for bing and spotlight
- `domain/src/sources/bing.rs` - BingSource with BingImage/BingResponse serde structs, async fetch with retry
- `domain/src/sources/spotlight.rs` - SpotlightSource with deeply nested serde structs, async fetch with retry
- `domain/src/lib.rs` - Added sources module and re-exported BingSource, SpotlightSource

## Decisions Made
- Used `Wallpaper::new()` constructor instead of struct literal initialization for consistency with the builder pattern established in 01-01
- Used `source_name` field (not `source`) in SourceError variants to match the thiserror 2.0 fix from 01-01
- Kept `attribution_url` in serde structs (parsed from API but not mapped to Wallpaper) with `#[allow(dead_code)]`
- No new dependencies needed — async-trait, reqwest, tokio, serde all already in workspace from 01-01

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed SourceError field names in plan template**
- **Found during:** Task 2 (BingSource implementation)
- **Issue:** Plan's code template used `source: "Bing".to_string()` but SourceError defines `source_name` field (renamed in 01-01 to avoid thiserror conflict)
- **Fix:** Used `source_name` field in all SourceError variant constructions
- **Files modified:** domain/src/sources/bing.rs, domain/src/sources/spotlight.rs
- **Verification:** `cargo check --package damask-domain` passes with no errors

**2. [Rule 3 - Blocking] async-trait already present in domain Cargo.toml**
- **Found during:** Task 2 (BingSource implementation)
- **Issue:** Plan specified adding `async-trait = "0.1"` to domain/Cargo.toml, but it was already added in 01-01
- **Fix:** Skipped the dependency addition — already present
- **Verification:** `cargo check` confirmed async-trait is available

**3. [Rule 1 - Bug] Suppressed dead_code warnings for serde-only fields**
- **Found during:** Task 4 (final cargo check)
- **Issue:** Compiler warned about `attribution_url` field in BingImage and SpotlightImage structs — needed for serde deserialization but never read after parsing
- **Fix:** Added `#[allow(dead_code)]` to both structs
- **Files modified:** domain/src/sources/bing.rs, domain/src/sources/spotlight.rs
- **Committed in:** `7f8d70e` (Task 4 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 1 blocking, 1 bug)
**Impact on plan:** All auto-fixes were necessary for correctness. No scope creep. Execution was straightforward since domain types from 01-01 were well-designed.

## Issues Encountered
- Plan's code template had stale field names (`source` instead of `source_name`) from before the 01-01 thiserror fix — corrected inline

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- BingSource and SpotlightSource ready for use in UI layer (Phase 3) and integration testing (Phase 4)
- SourceRegistry can register both sources for unified wallpaper fetching
- HTTP client pattern (timeout, retry, error handling) established for any future source implementations

## Self-Check: PASSED

All 3 created files exist. All 4 task commits verified in git history. `cargo check --package damask-domain` passes with zero warnings.

---
*Phase: 01-core-engine*
*Completed: 2026-04-14*
