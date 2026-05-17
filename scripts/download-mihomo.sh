#!/bin/bash
set -e

VERSION="${1:-v1.19.6}"
BINDIR="$(dirname "$0")/../src-tauri/binaries"

mkdir -p "$BINDIR"

ARCH=$(uname -m)
echo "Downloading mihomo $VERSION..."

# ARM64 (Apple Silicon)
echo "Downloading arm64 binary..."
curl -L "https://github.com/MetaCubeX/mihomo/releases/download/$VERSION/mihomo-darwin-arm64-$VERSION.gz" -o /tmp/mihomo-arm64.gz
gunzip -f /tmp/mihomo-arm64.gz
mv /tmp/mihomo-arm64 "$BINDIR/mihomo-aarch64-apple-darwin"
chmod +x "$BINDIR/mihomo-aarch64-apple-darwin"

# x86_64 (Intel)
echo "Downloading amd64 binary..."
curl -L "https://github.com/MetaCubeX/mihomo/releases/download/$VERSION/mihomo-darwin-amd64-$VERSION.gz" -o /tmp/mihomo-amd64.gz
gunzip -f /tmp/mihomo-amd64.gz
mv /tmp/mihomo-amd64 "$BINDIR/mihomo-x86_64-apple-darwin"
chmod +x "$BINDIR/mihomo-x86_64-apple-darwin"

echo "Done! Binaries placed in $BINDIR"
ls -la "$BINDIR"/mihomo-*
