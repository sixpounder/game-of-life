# Game Of Life

![Application icon](./data/icons/hicolor/scalable/apps/io.github.sixpounder.GameOfLife.svg)

A simple Conway's game of life simulator for the Gnome desktop

## Installation

The easieast way to install is from Flathub.

<a href="https://flathub.org/apps/details/io.github.sixpounder.GameOfLife"><img src="https://flathub.org/assets/badges/flathub-badge-en.png" width="200"/></a>

### Using Gnome Builder

Just clone this repository and hit the play button. Builder 43 also let you one-click install
the application to your device.

### Build from sources

You will need the meson build system and flatpak builder, along with gtk4 and libadwaita devel libraries.

```bash
git clone <this repo> game-of-life
cd game-of-life
meson build --prefix=/usr/local
ninja -C build
```

## The icon kinda stinks

If anyone is willing to give this small app an icon that is not complete garbage like the one I made, PRs are open <3

(the source svg is in `data/icons/hicolor/scalable/apps`)
