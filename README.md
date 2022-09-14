# Game Of Life

A simple Conway's game of life simulator for the Gnome desktop

## Installation

The easieast way to install is from Flathub.

### Build from sources

You will need the meson build system and flatpak builder. Either use an IDE like
Gnome Builder or

```bash
git clone <this repo> game-of-life
cd game-of-life
meson build --prefix=/usr/local
ninja -C build
```
