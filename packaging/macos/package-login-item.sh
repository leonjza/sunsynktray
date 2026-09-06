#!/usr/bin/env bash
set -euo pipefail

target="${1:?usage: package-login-item.sh TARGET_TRIPLE}"
app="dist/SunTray.app"
helper="${app}/Contents/Library/LoginItems/SunTrayStartup.app"

mkdir -p "${helper}/Contents/MacOS"
cp "target/${target}/release/suntray-startup" "${helper}/Contents/MacOS/suntray-startup"
cp packaging/macos/login-item/Info.plist "${helper}/Contents/Info.plist"
chmod +x "${helper}/Contents/MacOS/suntray-startup"
