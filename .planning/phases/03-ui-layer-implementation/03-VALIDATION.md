---
phase: 3
slug: ui-layer-implementation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | none — workspace default |
| **Quick run command** | `cargo test -p app` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p app`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | UI-01 | — | N/A | unit | `cargo test -p app` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 1 | UI-02 | — | N/A | unit | `cargo test -p app` | ❌ W0 | ⬜ pending |
| 3-02-01 | 02 | 1 | UI-03, UI-04 | — | N/A | unit | `cargo test -p app` | ❌ W0 | ⬜ pending |
| 3-02-02 | 02 | 1 | UI-05 | — | Error toasts don't leak internal details | unit | `cargo test -p app` | ❌ W0 | ⬜ pending |
| 3-03-01 | 03 | 2 | UI-06 | — | N/A | unit | `cargo test -p app` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `app/Cargo.toml` — workspace member entry + relm4/gtk4/libadwaita dependencies
- [ ] `app/src/main.rs` — basic app skeleton with relm4::App
- [ ] Framework install: cargo build -p app

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Window renders on screen | UI-01 | Requires display server | Run `cargo run -p app`, verify window appears |
| Wallpaper thumbnail displays | UI-01 | Visual check | Verify preview area shows downloaded image |
| Toast messages appear | UI-05 | Requires GTK visual feedback | Trigger an error, verify toast at bottom of window |
| Settings window opens | UI-06 | Requires display server | Click settings, verify preferences window |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
