# Dupont

A native Linux desktop application for setting wallpapers from online sources. Built with Rust and GTK/libadwaita.

![GTK4](https://img.shields.io/badge/GTK-4-blue)
![GNOME](https://img.shields.io/badge/GNOME-49-4a86cf)
![Rust](https://img.shields.io/badge/Rust-stable-orange)
![License](https://img.shields.io/badge/License-GPL_3.0-green)

## Features

- Fetch wallpapers from **Bing Wallpaper of the Day** and **Microsoft Spotlight**
- Wallpaper preview with metadata (title, description, attribution)
- Configurable Bing market and Spotlight locale
- Currently only supports **GNOME** desktop environments
- Persistent settings across sessions
- Error toasts for network and runtime failures

## Install

### Arch Linux (AUR)

```bash
git clone https://aur.archlinux.org/dupont.git
cd dupont
makepkg -si
```

Then run: `dupont`

### Build from source

Requires Rust stable, GTK4, and libadwaita.

```bash
git clone https://github.com/parkiyong/dupont.git
cd dupont
cargo build --release
./target/release/dupont
```

### Flatpak

Download the latest release from [GitHub Releases](https://github.com/parkiyong/dupont/releases) and install:

```bash
flatpak install io.github.parkiyong.dupont-1.2.0.flatpak
```

Or build from source (requires flatpak-builder and GNOME 49 SDK):

```bash
flatpak install flathub org.gnome.Platform//49 org.gnome.Sdk//49 org.freedesktop.Sdk.Extension.rust-stable//25.08
cd flatpak
flatpak-builder --user --install --force-clean --default-branch=stable build-dir io.github.parkiyong.dupont.json
flatpak run io.github.parkiyong.dupont
```

## Usage

1. Launch the app: `dupont`
2. Select a wallpaper source (Bing or Spotlight) from the dropdown
3. Click **Refresh** to fetch and apply a new wallpaper
4. Open **Settings** to configure Bing market or Spotlight locale
5. The wallpaper persists across sessions

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| UI Toolkit | GTK4 / libadwaita (via relm4) |
| Async Runtime | Tokio |
| HTTP Client | reqwest |
| Architecture | Clean separation — `dupont-domain` (core engine) + `dupont-app` (GTK UI) |

## Credits

Inspired by [Damask](https://gitlab.gnome.org/subpop/damask) by Link Dupont — hence the name.

## License

[GPLv3](LICENSE)
