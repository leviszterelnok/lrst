#!/usr/bin/env bash
# Tree-sitter parser build for lrs, és telepítés a Neovim-be.
# Használat: ./build.sh
# Kell hozzá: node/npx, cc (gcc vagy clang).
set -euo pipefail
cd "$(dirname "$0")"

NVIM_PARSER_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/site/parser"
NVIM_QUERY_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nvim/queries/lrs"
TS="npx --yes tree-sitter-cli@0.23.2"

$TS generate
cc -shared -fPIC -O2 -I src src/parser.c -o lrs.so

mkdir -p "$NVIM_PARSER_DIR" "$NVIM_QUERY_DIR"
cp lrs.so "$NVIM_PARSER_DIR/lrs.so"
cp queries/highlights.scm "$NVIM_QUERY_DIR/highlights.scm"

echo "kész: $NVIM_PARSER_DIR/lrs.so"
echo "kész: $NVIM_QUERY_DIR/highlights.scm"
