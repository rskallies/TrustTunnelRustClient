#!/usr/bin/env bash
# Cross-compile for Windows x64 from macOS.
#
# Prerequisites (one-time):
#   brew install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Usage:
#   ./build-windows.sh                    # uses cached vpn_easy in /tmp/vpn_easy_tmp
#   VPN_EASY_LIB_DIR=/path/to/dir ./build-windows.sh

set -euo pipefail

VPN_EASY_LIB_DIR="${VPN_EASY_LIB_DIR:-/tmp/vpn_easy_tmp}"

# Download vpn_easy if not already present
if [[ ! -f "$VPN_EASY_LIB_DIR/libvpn_easy.a" ]]; then
  echo "==> Downloading vpn_easy artifact..."
  mkdir -p "$VPN_EASY_LIB_DIR"
  RUN_ID=$(gh run list \
    --repo rskallies/TrustTunnelClient \
    --workflow "Build vpn_easy.dll (Windows x64)" \
    --status success --limit 1 \
    --json databaseId --jq '.[0].databaseId')
  gh run download "$RUN_ID" \
    --repo rskallies/TrustTunnelClient \
    --name vpn_easy-windows-x64 \
    --dir "$VPN_EASY_LIB_DIR"

  echo "==> Generating MinGW import library..."
  xcrun nm "$VPN_EASY_LIB_DIR/vpn_easy.lib" 2>/dev/null \
    | grep -E ' T ' | awk '{print $3}' | grep -v '^_imp_' | sed 's/^_//' \
    | { echo "LIBRARY vpn_easy"; echo "EXPORTS"; while read -r sym; do echo "  $sym"; done; } \
    > "$VPN_EASY_LIB_DIR/vpn_easy.def"

  x86_64-w64-mingw32-dlltool \
    -d "$VPN_EASY_LIB_DIR/vpn_easy.def" \
    -l "$VPN_EASY_LIB_DIR/libvpn_easy.a"
fi

echo "==> Building frontend..."
(cd ui && npm ci --silent && npm run build)

echo "==> Building Rust (release)..."
VPN_EASY_LIB_DIR="$VPN_EASY_LIB_DIR" \
  cargo build --release --target x86_64-pc-windows-gnu

echo ""
echo "Built:"
ls -lh target/x86_64-pc-windows-gnu/release/trusttunnel-service.exe \
        target/x86_64-pc-windows-gnu/release/trusttunnel.exe \
        target/x86_64-pc-windows-gnu/release/trusttunnel-installer.exe
