#!/bin/sh
# Point extension.toml's grammar `rev` at a tree-sitter-wyrm commit.
#
#   scripts/pin-grammar.sh path/to/tree-sitter-wyrm           # its HEAD, public URL
#   scripts/pin-grammar.sh path/to/tree-sitter-wyrm --local   # its HEAD, that checkout (file://)
#
# Gram/Zed compiles the grammar from a git repository at a fixed commit, so a
# grammar change only reaches the extension once the pin moves. Workflow:
#
#   (in tree-sitter-wyrm) commit, push
#   scripts/pin-grammar.sh path/to/tree-sitter-wyrm
#   git commit -am "Pin grammar to <rev>"
#
# The public URL only has pushed commits. To try unpushed grammar changes with
# Install Dev Extension, pin with --local, and don't commit that pin.
set -eu
usage() { echo "usage: scripts/pin-grammar.sh path/to/tree-sitter-wyrm [--local]" >&2; exit 2; }
[ $# -ge 1 ] || usage
grammar=$(cd "$1" && pwd)
public="https://github.com/Wyrm-Language/tree-sitter-wyrm"
case "${2:-}" in
  --local) repo="file://$grammar" ;;
  "") repo=$public ;;
  *) usage ;;
esac
[ -f "$grammar/grammar.js" ] || { echo "no grammar.js in $grammar" >&2; exit 2; }
rev=$(git -C "$grammar" rev-parse HEAD)
cd "$(dirname "$0")/.."
# Only the [grammars.wyrm] table: the top-level `repository` is the extension's
# own homepage.
sed -i -E \
  -e "/^\[grammars\.wyrm\]/,/^\[/{s|^rev = .*|rev = \"$rev\"|;s|^repository = .*|repository = \"$repo\"|}" \
  extension.toml
echo "extension.toml: grammar pinned to $rev ($repo)"
