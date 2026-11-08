#!/usr/bin/env bash
# Builds the Windows installer in the `tauri-build` Docker image (Linux, cross-compiling with
# cargo-xwin and NSIS), so nothing has to be installed on the host. The setup lands in release/.
#
#   npm run dist                 # from the repository root (Git Bash or WSL on Windows)
#
# Named volumes keep the Rust target, the cargo registry, the MSVC SDK (xwin) and the Linux
# node_modules between builds.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Docker Desktop on Windows needs a Windows path for the bind mount
if command -v cygpath >/dev/null 2>&1; then MOUNT="$(cygpath -w "$ROOT")"; else MOUNT="$ROOT"; fi
export MSYS_NO_PATHCONV=1

docker run --rm \
  -v "$MOUNT:/app" \
  -v pccleaner-target:/target \
  -v pccleaner-cargo:/usr/local/cargo/registry \
  -v pccleaner-xwin:/xwin \
  -v pccleaner-node-modules:/app/node_modules \
  -w /app \
  tauri-build bash -c '
    set -euo pipefail
    npm ci --no-audit --no-fund
    npm test
    (cd src-tauri && cargo test -p pccleaner-core)
    npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
    VERSION=$(node -p "require(\"./package.json\").version")
    mkdir -p release
    cp "/target/x86_64-pc-windows-msvc/release/bundle/nsis/PC Cleaner_${VERSION}_x64-setup.exe" "release/PCCleaner-Setup-${VERSION}.exe"
    cp /target/x86_64-pc-windows-msvc/release/PCCleaner.exe "release/PCCleaner.exe"
    ls -la release
  '
