#!/usr/bin/env sh
#
# Build wasabi and put it somewhere your shell will find it.
#
#   ./install.sh              build and install
#   ./install.sh --uninstall  remove it again
#
# Installs to ~/.cargo/bin when that exists, since anyone with a Rust
# toolchain already has it on PATH, and falls back to ~/.local/bin otherwise.
# Override with:
#
#   INSTALL_DIR=/usr/local/bin ./install.sh

set -eu

BIN=wasabi

# Work from the project directory, so the script runs from anywhere.
cd "$(dirname "$0")"

# --- where does it go? -------------------------------------------------------

if [ -z "${INSTALL_DIR:-}" ]; then
    if [ -d "$HOME/.cargo/bin" ]; then
        INSTALL_DIR="$HOME/.cargo/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
    fi
fi

TARGET="$INSTALL_DIR/$BIN"

# --- uninstall ---------------------------------------------------------------

if [ "${1:-}" = "--uninstall" ]; then
    if [ -e "$TARGET" ]; then
        rm -f "$TARGET"
        echo "removed $TARGET"
        echo
        echo "your records, settings and themes were left alone, in"
        echo "  ${XDG_DATA_HOME:-$HOME/.local/share}/$BIN"
    else
        echo "nothing installed at $TARGET"
    fi
    exit 0
fi

if [ "${1:-}" != "" ]; then
    echo "usage: $0 [--uninstall]" >&2
    exit 2
fi

# --- build -------------------------------------------------------------------

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found — install Rust from https://rustup.rs" >&2
    exit 1
fi

echo "building $BIN..."
cargo build --release

BUILT="target/release/$BIN"
if [ ! -x "$BUILT" ]; then
    echo "error: expected a binary at $BUILT but found none" >&2
    exit 1
fi

# --- install -----------------------------------------------------------------

mkdir -p "$INSTALL_DIR"

# Copy to a temporary name first, then move into place. A plain cp over a
# binary that is currently running can fail; a rename never does.
cp "$BUILT" "$TARGET.new"
chmod 755 "$TARGET.new"
mv -f "$TARGET.new" "$TARGET"

echo "installed $TARGET ($(du -h "$TARGET" | cut -f1))"

# --- is it actually reachable? -----------------------------------------------

case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        echo
        echo "run it with:  $BIN"
        ;;
    *)
        echo
        echo "warning: $INSTALL_DIR is not on your PATH, so '$BIN' won't run yet."
        echo "add it by running:"
        echo
        case "${SHELL:-}" in
            *zsh)  echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc && exec zsh" ;;
            *bash) echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc && exec bash" ;;
            *)     echo "  export PATH=\"$INSTALL_DIR:\$PATH\"   # add this to your shell's rc file" ;;
        esac
        ;;
esac
