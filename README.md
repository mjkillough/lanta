# lanta — [![Build Status](https://travis-ci.org/mjkillough/lanta.svg?branch=master)](https://travis-ci.org/mjkillough/lanta)

Experiments in creating a tiling X11 window manager in Rust.

Lanta is written to be customisable, simple and fast-ish.


## Features

Lanta doesn't implement all of [EWMH](https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html) or [ICCCM](https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html), nor will it ever. It aims to implement just enough for use as my primary WM.

At the core of Lanta is its groups (somestimes called 'workspaces' by other WMs) and each group has a stack of windows. Windows can be moved between groups, can be focused inside a group and can be shuffled up/down within the group's stack. Each group has a set of layouts which control how the stack of groups is shown on the screen and a group's layout can be altered at run-time.

There are currently a few simple layouts implemented:

 - Stack — Maximises the currently focused window.
 - Tiled — Shows all windows in the group's stack vertically.

... but if you look at `src/layouts.rs` you should see it's easy to add more.


## Installing

Lanta currently requires the nightly version of Rust to compile. It should be relatively easy to port it to work on the stable version of Rust if required.

Your system must first have all of the required [dependencies](#dependencies)

To accept the default key shortcuts, layouts and groups, you can install and run using:

```sh
cargo install lanta
# Run directly or add to your .xinitrc:
lanta
```

However, the default configuration is almost certainly not what you want.

You should either clone this repository and modify `src/bin/lanta.rs` to your liking, or (preferably) make a new binary project which depends on `lanta`. The code in `src/bin/lanta.rs` should give you an idea of what to do in your binary project.


## Dependencies

In addition to the Rust dependencies in `Cargo.toml`, Lanta also depends on these system libraries:

 - `x11-xcb`
 - `xcb-util`: `xcb-ewmh` / `xcb-icccm` / `xcb-keysyms`

The following Ubuntu packages should allow your system to meet these requirements:

```sh
sudo apt-get install -y libx11-xcb-dev libxcb-ewmh-dev libxcb-icccm4-dev libxcb-keysyms1-dev
```

Lanta currently depends on some unreleased/custom patches in the following Rust projects: `xcb`. This won't be the case forever.


## Default Configuration

In the default configuration, the following short-cuts are available:

 - `Mod4 + a` / `Mod4 + s` / `Mod4 + d` / `Mod4 + f` — Switch between groups.
 - `Mod4 + Shift + a` / `Mod4 + Shift + s` / `Mod4 + Shift + d` / `Mod4 + Shift + f` — Move currently focused window to the specified group.
 - `Mod4 + j` — Switch focus to the next window in the current group's stack.
 - `Mod4 + k` — Switch focus to the previous window in the current group's stack.
 - `Mod4 + Shift + j` — Shuffle the currently focused window up in the current group's stack.
 - `Mod4 + Shift + k` — Shuffle the currently focused window down in the current group's stack.
 - `Mod4 + Tab` — Switch the layout in the current group.
 - `Mod4 + Return` — Open a terminal (`urxvt`).
 - `Mod4 + c` — Open Google Chrome.
 - `Mod4 + v` — Open Visual Studio Code
 - `Mod4 + q` — Change wallpaper using my [`change-wallpaper`](https://github.com/mjkillough/change-wallpaper) script.

... where `Mod4` is usually the Cmd/Windows/Super key. As described in the [installation section](#installing), it's expected you'll make your own configuration, rather than using mine.


## Tests

Unit tests:

```sh
cargo test
```

... which are run on Travis CI for every commit.

Integration tests using `Xephyr` are yet to be implemented.


## License

MIT
