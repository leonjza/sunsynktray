#!/usr/bin/env bash
set -euo pipefail

target="${1:?usage: package.sh TARGET_TRIPLE}"

binary="target/${target}/release/suntray"
if [[ ! -x "${binary}" ]]; then
  cargo build --locked --release --target "${target}"
fi
cargo packager --release --target "${target}" --formats app
bash packaging/macos/package-login-item.sh "${target}"
