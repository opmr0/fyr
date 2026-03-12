#!/bin/bash
set -e

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="aarch64"
else
    ARCH="x86_64"
fi

case "$OS" in
    linux)  BINARY="fyr-linux-x86_64" ;;
    darwin)
        if [ "$ARCH" = "aarch64" ]; then
            BINARY="fyr-macos-aarch64"
        else
            BINARY="fyr-macos-x86_64"
        fi
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Installing fyr for $OS ($ARCH)..."

LATEST=$(curl -s https://api.github.com/repos/opmr0/fyr/releases/latest | grep '"tag_name"' | head -1 | cut -d '"' -f 4)

if [ -z "$LATEST" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

echo "Downloading $LATEST..."
curl -L -o /tmp/fyr "https://github.com/opmr0/fyr/releases/download/$LATEST/$BINARY"
chmod +x /tmp/fyr

echo "Installing to /usr/local/bin (may require sudo)..."
sudo mv /tmp/fyr /usr/local/bin/fyr

echo ""
echo "fyr installed successfully!"
echo "Run 'fyr --help' to get started"
