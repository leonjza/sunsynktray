# sunsynktray

[![API Smoke Test](https://github.com/leonjza/sunsynktray/actions/workflows/api-smoke.yml/badge.svg)](https://github.com/leonjza/sunsynktray/actions/workflows/api-smoke.yml)

SunSynk Tray Applications for Inverter Monitoring using the SunSynk Cloud Service.

## Packaging

Packaging is managed by [`cargo-packager`](https://docs.rs/cargo-packager/latest/cargo_packager/).

Install it once with:

```sh
cargo install cargo-packager --locked
```

Build the release binary for the target platform, then run `cargo packager --release`. macOS produces the application bundle; the release workflow wraps it in a native `.pkg` installer. Windows produces a current-user NSIS installer, and the release workflow also publishes a portable Windows ZIP.

## Core dependency notes

SunTray was migrated to `gpui-kit` 0.6 and its matching `gpui-pre` platform
layer. The migration updated the application and component APIs, theme
initialisation, background timers, and chart configuration while preserving
system light/dark appearance.

`gpui-tray` is vendored in [`vendor/gpui-tray`](vendor/gpui-tray) and selected
through the workspace patch in `Cargo.toml`. The local version follows the
GPUI 0.6-era `gpui-pre` API and includes these platform changes:

- macOS supports native SF Symbols and native status-item titles, keeping
  numeric tray values sharp on Retina displays.
- Windows converts RGBA icons into native Windows icons with an alpha mask,
  allowing the tray to display readable, DPI-aware metric values.
- Tray menu actions remain regular GPUI actions, and tray resources are kept
  on the GPUI thread with explicit cleanup.

These dependency changes should be validated with both Windows and macOS
packaging builds.
