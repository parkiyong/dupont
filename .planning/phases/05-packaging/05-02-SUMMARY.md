---
plan: 05-02
phase: 05-packaging
status: complete
completed: 2026-04-19
---

# Phase 5 Plan 2 Summary: AUR Package

## Objective
Create an Arch Linux AUR package (PKGBUILD) that builds and installs Dupont.

## Implementation

### Files Created
- `PKGBUILD` — Arch package build script

### Key Details
- Source build from git tag (v0.1.0)
- Binary installed as `/usr/bin/dupont-app` (matches desktop file Exec=)
- Desktop file and icons installed to standard XDG paths
- `prepare()` runs `cargo fetch --locked` for offline build compliance
- `build()` uses `--frozen --release` for reproducible builds
- `!lto` option to avoid Rust LTO build issues on Arch
- License: GPL-3.0-or-later

## Verification
- JSON manifest validated with python3
- All install paths verified against actual file locations
- Binary name matches desktop file Exec= entry
- namcap not available for linting

## Install Instructions
```bash
git clone https://aur.archlinux.org/dupont.git
cd dupont
makepkg -si
```

## Notes
- flatpak-builder not installed, could not test full Flatpak build
- Requires `cargo` in makedepends (standard for Rust AUR packages)
- No -bin variant created (source build only)
