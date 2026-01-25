#!/bin/bash
# GlowBarn Installer - bad-antics
set -e
PREFIX="${PREFIX:-/usr/local}"
install -Dm755 glowbarn "$PREFIX/bin/glowbarn"
install -Dm644 glowbarn.desktop "$PREFIX/share/applications/glowbarn.desktop"
echo "✅ GlowBarn installed to $PREFIX/bin/glowbarn"
