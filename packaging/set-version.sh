#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:?usage: set-version.sh VERSION}"

update_cargo_version() {
    local input="$1"
    local output="${input}.tmp"

    awk -v version="$VERSION" '
        !updated && $0 ~ /^version = "[^"]*"$/ {
            sub(/"[^"]*"$/, "\"" version "\"")
            updated = 1
        }
        { print }
        END {
            if (!updated) exit 1
        }
    ' "$input" > "$output"
    mv "$output" "$input"
}

update_plist_versions() {
    local input="$1"
    local output="${input}.tmp"

    awk -v version="$VERSION" '
        /<key>CFBundleShortVersionString<\/key>/ || /<key>CFBundleVersion<\/key>/ {
            print
            if (getline <= 0) exit 1
            sub(/<string>[^<]*<\/string>/, "<string>" version "</string>")
            print
            next
        }
        { print }
    ' "$input" > "$output"
    mv "$output" "$input"
}

update_cargo_version "$PROJECT_ROOT/Cargo.toml"
update_plist_versions "$PROJECT_ROOT/packaging/macos/Info.plist"

echo "Set application version to ${VERSION}"
