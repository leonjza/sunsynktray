# sunsynktray

[![API Smoke Test](https://github.com/leonjza/sunsynktray/actions/workflows/api-smoke.yml/badge.svg)](https://github.com/leonjza/sunsynktray/actions/workflows/api-smoke.yml)

SunSynk Tray Applications for Inverter Monitoring using the SunSynk Cloud Service.

## Packaging

Packaging is managed by [`cargo-packager`](https://docs.rs/cargo-packager/latest/cargo_packager/).

Install it once with:

```sh
cargo install cargo-packager --locked
```

Build the release binary for the target platform, then run `cargo packager --release`. macOS produces the application bundle and DMG; Windows produces a current-user NSIS installer. The release workflow also publishes a portable Windows ZIP.
