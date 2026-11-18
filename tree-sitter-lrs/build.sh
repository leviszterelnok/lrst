#!/usr/bin/env bash
# Tree-sitter parser build for lrs, és telepítés a Neovim-be.
# Használat: ./build.sh
# Kell hozzá: tree-sitter CLI (PATH-on), cc (gcc vagy clang).
#   telepítés: cargo install tree-sitter-cli   vagy   npm install -g tree-sitter-cli
set -euo pipefail
cd "$(dirname "$0")"

NVIM_PARSER_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/site/parser"
NVIM_QUERY_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nvim/queries/lrs"

if command -v tree-sitter >/dev/null 2>&1; then
  TS=(tree-sitter)
else
  echo "A 'tree-sitter' parancs nincs a PATH-on. Telepítsd:" >&2
  echo "  cargo install tree-sitter-cli   (vagy: npm install -g tree-sitter-cli)" >&2
  exit 1
fi

"${TS[@]}" generate
cc -shared -fPIC -O2 -I src src/parser.c -o lrs.so

mkdir -p "$NVIM_PARSER_DIR" "$NVIM_QUERY_DIR"
cp lrs.so "$NVIM_PARSER_DIR/lrs.so"
cp queries/highlights.scm queries/injections.scm "$NVIM_QUERY_DIR/"

echo "kész: $NVIM_PARSER_DIR/lrs.so"
echo "kész: $NVIM_QUERY_DIR/{highlights,injections}.scm"
