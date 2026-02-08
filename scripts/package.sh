#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-}"
OUT_DIR="${2:-dist}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

pushd "$ROOT" > /dev/null

if [[ -n "$TARGET" ]]; then
  cargo build --release --target "$TARGET"
  BIN="target/$TARGET/release/a"
  PKG="a-$TARGET"
else
  cargo build --release
  BIN="target/release/a"
  OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
  case "$OS" in
    darwin) OS="macos" ;;
    linux) OS="linux" ;;
  esac
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
  esac
  PKG="a-${OS}-${ARCH}"
fi

if [[ ! -f "$BIN" ]]; then
  echo "Expected binary not found: $BIN" >&2
  exit 1
fi

STAGE_BASE="$OUT_DIR/_stage"
STAGE="$STAGE_BASE/$PKG"
TAR="$OUT_DIR/$PKG.tar.gz"
RAW="$OUT_DIR/$PKG"

rm -rf "$STAGE"
rm -f "$TAR"
rm -f "$RAW"
mkdir -p "$STAGE/bin"

cp "$BIN" "$STAGE/bin/a"
chmod +x "$STAGE/bin/a"
cp "$ROOT/README.md" "$STAGE/README.md"
cp "$ROOT/syntax.md" "$STAGE/syntax.md"
cp "$ROOT/scripts/install.sh" "$STAGE/install.sh"
chmod +x "$STAGE/install.sh"

tar -czf "$TAR" -C "$STAGE" .
cp "$BIN" "$RAW"
chmod +x "$RAW"

popd > /dev/null

echo "Wrote $TAR"
echo "Wrote $RAW"
