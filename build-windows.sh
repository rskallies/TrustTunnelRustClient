#!/usr/bin/env bash
# Cross-compile for Windows x64 from macOS and stage a release directory.
#
# Prerequisites (one-time):
#   brew install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Usage:
#   ./build-windows.sh                    # uses cached artifacts in /tmp/tt-deps
#   VPN_EASY_LIB_DIR=/path/to/dir ./build-windows.sh

set -euo pipefail

DEPS_DIR="${VPN_EASY_LIB_DIR:-/tmp/tt-deps}"
WINTUN_URL="https://www.wintun.net/builds/wintun-0.14.1.zip"
WEBVIEW2_VERSION="1.0.3800.47"
WEBVIEW2_URL="https://api.nuget.org/v3-flatcontainer/microsoft.web.webview2/${WEBVIEW2_VERSION}/microsoft.web.webview2.${WEBVIEW2_VERSION}.nupkg"
RELEASE_DIR="release"

# ── vpn_easy ────────────────────────────────────────────────────────────────
if [[ ! -f "$DEPS_DIR/libvpn_easy.a" ]]; then
  echo "==> Downloading vpn_easy artifact..."
  mkdir -p "$DEPS_DIR"
  RUN_ID=$(gh run list \
    --repo rskallies/TrustTunnelClient \
    --workflow "Build vpn_easy.dll (Windows x64)" \
    --status success --limit 1 \
    --json databaseId --jq '.[0].databaseId')
  gh run download "$RUN_ID" \
    --repo rskallies/TrustTunnelClient \
    --name vpn_easy-windows-x64 \
    --dir "$DEPS_DIR"

  echo "==> Generating MinGW import library..."
  xcrun nm "$DEPS_DIR/vpn_easy.lib" 2>/dev/null \
    | grep -E ' T ' | awk '{print $3}' | grep -v '^_imp_' | sed 's/^_//' \
    | { echo "LIBRARY vpn_easy"; echo "EXPORTS"; while read -r sym; do echo "  $sym"; done; } \
    > "$DEPS_DIR/vpn_easy.def"

  x86_64-w64-mingw32-dlltool \
    -d "$DEPS_DIR/vpn_easy.def" \
    -l "$DEPS_DIR/libvpn_easy.a"
fi

# ── wintun ───────────────────────────────────────────────────────────────────
if [[ ! -f "$DEPS_DIR/wintun.dll" ]]; then
  echo "==> Downloading wintun..."
  curl -sSL "$WINTUN_URL" -o "$DEPS_DIR/wintun.zip"
  unzip -q "$DEPS_DIR/wintun.zip" -d "$DEPS_DIR/wintun_tmp"
  cp "$DEPS_DIR/wintun_tmp/wintun/bin/amd64/wintun.dll" "$DEPS_DIR/wintun.dll"
  rm -rf "$DEPS_DIR/wintun_tmp" "$DEPS_DIR/wintun.zip"
fi

# ── WebView2Loader.dll ───────────────────────────────────────────────────────
if [[ ! -f "$DEPS_DIR/WebView2Loader.dll" ]]; then
  echo "==> Downloading WebView2Loader.dll..."
  curl -sSL "$WEBVIEW2_URL" -o "$DEPS_DIR/webview2.nupkg"
  python3 -c "
import zipfile, sys
z = zipfile.ZipFile('$DEPS_DIR/webview2.nupkg')
data = z.read('runtimes/win-x64/native/WebView2Loader.dll')
open('$DEPS_DIR/WebView2Loader.dll', 'wb').write(data)
"
  rm "$DEPS_DIR/webview2.nupkg"
fi

# ── Frontend ─────────────────────────────────────────────────────────────────
echo "==> Building frontend..."
(cd ui && npm ci --silent && npm run build)

# ── Rust ─────────────────────────────────────────────────────────────────────
echo "==> Building Rust (release)..."
VPN_EASY_LIB_DIR="$DEPS_DIR" \
  cargo build --release --target x86_64-pc-windows-gnu

# ── Stage release directory ───────────────────────────────────────────────────
echo "==> Staging release directory..."
mkdir -p "$RELEASE_DIR"
cp target/x86_64-pc-windows-gnu/release/trusttunnel-service.exe   "$RELEASE_DIR/"
cp target/x86_64-pc-windows-gnu/release/trusttunnel.exe           "$RELEASE_DIR/"
cp target/x86_64-pc-windows-gnu/release/trusttunnel-installer.exe "$RELEASE_DIR/"
cp "$DEPS_DIR/vpn_easy.dll"         "$RELEASE_DIR/"
cp "$DEPS_DIR/wintun.dll"           "$RELEASE_DIR/"
cp "$DEPS_DIR/WebView2Loader.dll"   "$RELEASE_DIR/"

echo ""
echo "Release directory: $RELEASE_DIR/"
ls -lh "$RELEASE_DIR/"
