#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET="aarch64-apple-darwin"
APP_NAME="SunTray"
EXECUTABLE="suntray"
VERSION="${VERSION:-0.1.0}"
DIST_DIR="${PROJECT_ROOT}/dist/macos"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"
DMG_PATH="${DIST_DIR}/${APP_NAME}-${VERSION}-macos-arm64.dmg"
STAGING_DIR="$(mktemp -d "${TMPDIR:-/tmp}/suntray-dmg.XXXXXX")"

cleanup() {
    rm -rf "${STAGING_DIR}"
}
trap cleanup EXIT

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "This script must run on macOS because it uses hdiutil." >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx "${TARGET}"; then
    echo "Rust target ${TARGET} is not installed." >&2
    echo "Install it with: rustup target add ${TARGET}" >&2
    exit 1
fi

mkdir -p "${DIST_DIR}"
rm -rf "${APP_DIR}" "${DMG_PATH}"

echo "Building ${APP_NAME} for Apple Silicon…"
cargo build --release --target "${TARGET}" --manifest-path "${PROJECT_ROOT}/Cargo.toml"

mkdir -p "${APP_DIR}/Contents/MacOS" "${APP_DIR}/Contents/Resources"
cp "${PROJECT_ROOT}/target/${TARGET}/release/${EXECUTABLE}" "${APP_DIR}/Contents/MacOS/${EXECUTABLE}"
cp "${PROJECT_ROOT}/packaging/macos/Info.plist" "${APP_DIR}/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString ${VERSION}" "${APP_DIR}/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion ${VERSION}" "${APP_DIR}/Contents/Info.plist"

# Keep the bundle usable without an icon asset. Add SunTray.icns when available.
if [[ -f "${PROJECT_ROOT}/packaging/macos/SunTray.icns" ]]; then
    cp "${PROJECT_ROOT}/packaging/macos/SunTray.icns" "${APP_DIR}/Contents/Resources/SunTray.icns"
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string SunTray" "${APP_DIR}/Contents/Info.plist"
else
    echo "Note: packaging/macos/SunTray.icns not found; using the default application icon." >&2
fi

chmod 755 "${APP_DIR}/Contents/MacOS/${EXECUTABLE}"

mkdir -p "${STAGING_DIR}/${APP_NAME}"
ditto "${APP_DIR}" "${STAGING_DIR}/${APP_NAME}/${APP_NAME}.app"
ln -s /Applications "${STAGING_DIR}/${APP_NAME}/Applications"

echo "Creating ${DMG_PATH}…"
hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${STAGING_DIR}/${APP_NAME}" \
    -ov \
    -format UDZO \
    "${DMG_PATH}" >/dev/null

echo "Created: ${DMG_PATH}"
echo "Unsigned distribution: users may need to approve the app in Privacy & Security."
