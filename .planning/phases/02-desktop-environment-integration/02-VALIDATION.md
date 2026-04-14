---
phase: 2
slug: desktop-environment-integration
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-15
---

# Phase 2 -- Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]` macro, `#[cfg(test)]` modules) |
| **Config file** | none -- uses default Cargo test runner |
| **Quick run command** | `cargo test -p damask-domain` |
| **Full suite command** | `cargo test -p damask-domain` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p damask-domain`
- **After every plan wave:** Run `cargo test -p damask-domain`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~2 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|--------|
| 02-01-01 | 01 | 1 | DESK-01 | -- | DE detection parses XDG_CURRENT_DESKTOP correctly | unit | `cargo test -p damask-domain -- desktop::tests::detect_desktop_environment_parsing_and_routing` | green |
| 02-01-02 | 01 | 1 | DESK-02 | T-02-01 T-02-02 T-02-03 | GnomeDE sets wallpaper via gio::Settings | manual-only | -- | manual |
| 02-02-01 | 02 | 2 | DESK-03 | T-02-04 T-02-05 T-02-06 | CosmicDE writes RON config, reads back path | unit | `cargo test -p damask-domain -- desktop::cosmic::tests` | green |
| 02-01-01 | 01 | 1 | DESK-04 | -- | DEError variants produce clear human-readable messages | unit | `cargo test -p damask-domain -- error::tests` | green |

*Status: pending -- green -- red -- flaky*

---

## Wave 0 Requirements

Existing inline test modules cover all testable phase requirements. No separate test files or framework installation needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| GnomeDE.set_wallpaper actually changes desktop background | DESK-02 | Requires GNOME runtime with GSettings schema | Run app on GNOME system, call `GnomeDE.set_wallpaper("/path/to/image.jpg")`, verify background changes |
| GnomeDE.get_current_wallpaper reads current wallpaper from GSettings | DESK-02 | Requires GNOME runtime with GSettings schema | Set wallpaper via GNOME Settings, call `GnomeDE.get_current_wallpaper()`, verify correct path returned |

---

## Validation Sign-Off

- [x] All tasks have automated verify or manual-only justification
- [x] Sampling continuity: no 3 consecutive tasks without verification
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter (DESK-02 manual-only is justified by GNOME runtime dependency)

**Approval:** approved 2026-04-15

---

## Validation Audit 2026-04-15

| Metric | Count |
|--------|-------|
| Gaps found | 4 |
| Resolved (automated) | 3 |
| Manual-only (justified) | 1 |
| Escalated | 0 |

---
*Created: 2026-04-15*
