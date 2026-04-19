# Dupont

A native Linux desktop application for setting wallpapers from online sources. Built with Rust and GTK/libadwaita.

![GTK4](https://img.shields.io/badge/GTK-4-blue)
![Rust](https://img.shields.io/badge/Rust-stable-orange)
![License](https://img.shields.io/badge/License-Apache_2.0-green)

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

### Build from source

Requires Rust stable, GTK4, and libadwaita.

```bash
git clone https://github.com/parkiyong/dupont.git
cd dupont
cargo build --release
./target/release/dupont-app
```

### Flatpak (manual build)

```bash
cd flatpak
flatpak-builder --user --install --force-clean repo io.github.parkiyong.dupont.json
flatpak run io.github.parkiyong.dupont
```

## Usage

1. Select a wallpaper source (Bing or Spotlight) from the dropdown
2. Click **Refresh** to fetch and apply a new wallpaper
3. Open **Settings** to configure Bing market or Spotlight locale

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| UI Toolkit | GTK4 / libadwaita (via relm4) |
| Async Runtime | Tokio |
| HTTP Client | reqwest |
| Architecture | Clean separation — `dupont-domain` (core engine) + `dupont-app` (GTK UI) |

## License

[Apache License 2.0](LICENSE)
