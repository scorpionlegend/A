#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$ROOT/bin/a"

SCOPE="user"
for arg in "$@"; do
  case "$arg" in
    --scope=*)
      SCOPE="${arg#*=}"
      ;;
    --scope)
      shift
      SCOPE="${1:-}"
      ;;
    --system)
      SCOPE="system"
      ;;
    --user)
      SCOPE="user"
      ;;
  esac
done

if [[ "$SCOPE" != "user" && "$SCOPE" != "system" ]]; then
  echo "Invalid scope: $SCOPE (use --scope user|system)" >&2
  exit 1
fi

if [[ ! -f "$SRC" ]]; then
  echo "Could not find $SRC. Run this script from the extracted package root." >&2
  exit 1
fi

if [[ "$SCOPE" == "system" ]]; then
  DEST="/usr/local/bin"
  if [[ "$(id -u)" -ne 0 ]]; then
    echo "System install requires admin privileges. Re-run with sudo." >&2
    exit 1
  fi
else
  DEST="${HOME}/.local/bin"
fi
mkdir -p "$DEST"

cp "$SRC" "$DEST/a"
chmod +x "$DEST/a"

if [[ "$SCOPE" == "user" ]]; then
  if [[ ":$PATH:" != *":$DEST:"* ]]; then
    PROFILE="${HOME}/.profile"
    if ! grep -q "$DEST" "$PROFILE" 2>/dev/null; then
      echo "export PATH=\"$DEST:\$PATH\"" >> "$PROFILE"
    fi
    export PATH="$DEST:$PATH"
    echo "Added $DEST to PATH (via $PROFILE)."
  fi
else
  echo "Installed to $DEST. Ensure /usr/local/bin is on your PATH."
fi

echo "Installed A to $DEST"
echo "Open a new terminal and run: a --help"
