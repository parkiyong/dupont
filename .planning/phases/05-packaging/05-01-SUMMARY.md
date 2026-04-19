---
plan: 05-01
phase: 05-packaging
status: complete
completed: 2026-04-19
---

# Phase 5 Plan 1 Summary: Flatpak Package

## Objective
Create a Flatpak package that builds and runs Dupont with proper sandbox permissions.

## Implementation

### Files Created
- `flatpak/io.github.parkiyong.dupont.json` — Flatpak build manifest
- `flatpak/io.github.parkiyong.dupont.yml` — flatpak-builder build definition

### Key Details
- Runtime: org.gnome.Platform 47 (current stable)
- SDK: org.gnome.Sdk 47
- Sandbox permissions: network, config dir (~/.config/dupont), cache dir (~/.cache/dupont), X11 fallback, DRI GPU
- Two modules: rust (build), dupont (install)
- Offline build with `--offline --frozen` for reproducibility
- Cleanup: removes headers (*.a), debug symbols, /include

## Verification
- JSON manifest validated as syntactically correct
- YAML build definition syntactically correct
- All asset file paths verified against actual files
- flatpak-builder not installed — could not run full build test

## Build Instructions
```bash
# Install flatpak-builder
flatpak install org.gnome.Sdk//47 org.gnome.Platform//47
pip install flatpak-builder

# Build (from project root)
cd flatpak
flatpak-builder --user --install --force-clean repo io.github.parkiyong.dupont.json

# Run
flatpak run io.github.parkiyong.dupont
```

## Notes
- flatpak-builder not installed on system — manifest structure follows flathub conventions
- Not tested end-to-end (requires flatpak-builder + GNOME 47 SDK)
- Future: submit to Flathub once tested
