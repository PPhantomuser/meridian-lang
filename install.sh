#!/bin/sh
set -e

# --- Configuration ---
REPO="PPhantomuser/meridian-lang"
INSTALL_DIR="$HOME/.meridian/bin"
BIN_NAME="meridian"
# ---------------------

# Print beautiful banner
echo ""
echo "    __  ___           _     ___"
echo "   /  |/  /__________(_)___/ (_)___ _____ "
echo "  / /|_/ / _ \ ___/ / / __  / / __ \`/ __ \\"
echo " / /  / /  __/ /  / / / /_/ / / /_/ / / / /"
echo "/_/  /_/\___/_/  /_/_/\__,_/_/\__,_/_/ /_/ "
echo ""
echo "Welcome to the Meridian installer!"
echo ""

# OS/Arch detection
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$ARCH" = "x86_64" ] || [ "$ARCH" = "amd64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

if [ "$OS" = "darwin" ]; then
    OS="mac"
elif [ "$OS" = "linux" ]; then
    OS="linux"
elif echo "$OS" | grep -q "mingw\|msys\|cygwin"; then
    OS="windows"
    # Windows releases have .exe extension
    RELEASE_FILE="${BIN_NAME}-${OS}-${ARCH}.exe"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${RELEASE_FILE}"
    BIN_NAME="${BIN_NAME}.exe"
else
    echo "Unsupported operating system: $OS"
    exit 1
fi

if [ "$OS" != "windows" ]; then
    RELEASE_FILE="${BIN_NAME}-${OS}-${ARCH}"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${RELEASE_FILE}"
fi



echo ">> Downloading Meridian for ${OS} (${ARCH})..."

mkdir -p "$INSTALL_DIR"

if curl -sSfL "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BIN_NAME"; then
    chmod +x "$INSTALL_DIR/$BIN_NAME"
    echo ">> Successfully downloaded."
else
    echo "Error: Failed to download from $DOWNLOAD_URL"
    echo "Make sure the release exists on GitHub!"
    exit 1
fi

# Add to PATH
PROFILE_FILE=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    PROFILE_FILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
    PROFILE_FILE="$HOME/.bashrc"
else
    PROFILE_FILE="$HOME/.profile"
fi

if ! grep -q "$INSTALL_DIR" "$PROFILE_FILE" 2>/dev/null; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$PROFILE_FILE"
    echo ">> Added $INSTALL_DIR to your PATH in $PROFILE_FILE"
fi

echo ""
echo "Meridian was successfully installed!"
echo "To get started, please restart your terminal or run:"
echo "    source $PROFILE_FILE"
echo ""
echo "Then test your installation:"
echo "    meridian --version"
echo ""
