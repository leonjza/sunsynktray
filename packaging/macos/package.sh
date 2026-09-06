#!/usr/bin/env bash
set -euo pipefail

target="${1:?usage: package.sh TARGET_TRIPLE}"

cargo build --locked --release --target "${target}"
cargo packager --release --target "${target}" --formats app
bash packaging/macos/package-login-item.sh "${target}"
