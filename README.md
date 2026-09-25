# wyrm-gram

Gram (Zed-fork) editor extension for the [Wyrm](https://wyrm-lang.com) language.

| Feature | Provided by |
| --- | --- |
| Syntax highlighting, brackets, outline, indentation, text objects | the [tree-sitter-wyrm](https://github.com/Wyrm-Language/tree-sitter-wyrm) grammar, queries in `languages/wyrm/` |
| Diagnostics, go to definition, hover, completion, document symbols | `wyrm-lsp` (from [wypoc](https://github.com/Wyrm-Language/wypoc)) |
| Debugging (breakpoints, stepping) | `wyrm-dap` (from wypoc) |

The VS Code equivalent is [wyrm-vscode](https://github.com/Wyrm-Language/wyrm-vscode); the
language definition both follow is `doc/grammar.md` in the
[wyrm](https://codeberg.org/wyrm-lang/wyrm) repository.

## Install (development)

1. Rust needs the wasm target to build the extension (Arch: `sudo pacman -S rust-wasm`;
   with rustup: `rustup target add wasm32-wasip2`).
2. In Gram, open the Extensions view and choose **Install Dev Extension**, then pick this
   directory (the command is also in the command palette).

Gram downloads wasi-sdk on first use to compile the grammar.

## Configuration

`wyrm-lsp` and `wyrm-dap` are found in this order: the explicit setting, the project's
`.venv/bin`, then `PATH`.

```jsonc
// settings.json
"lsp": {
  "wyrm-lsp": {
    "binary": { "path": "/path/to/wyrm-lsp" },          // optional
    "settings": {
      // Module search path for `import` (WYRM_PATH). Relative entries are resolved
      // against the project root; an existing WYRM_PATH is appended unless
      // "module_path_inherit_environment" is false.
      "module_path": ["wy", "${worktree}/lib"]
    }
  }
}
```

The debugger uses the same environment, so `import`s resolve identically. Add a scenario to
`.gram/debug.json` (or start a session from the debug panel and pick the Wyrm adapter):

```json
[{ "label": "Debug current file", "adapter": "wyrm", "program": "$GRAM_FILE", "stopOnEntry": false }]
```

## Development

The grammar lives in [tree-sitter-wyrm](https://github.com/Wyrm-Language/tree-sitter-wyrm).
`extension.toml`'s `[grammars.wyrm]` pins it to a commit, and Gram clones and compiles that
commit; the queries in `languages/wyrm/` are read from this
repository and must match the pinned grammar's node names.

After a grammar change is committed and pushed there:

```sh
scripts/pin-grammar.sh path/to/tree-sitter-wyrm            # rev = its HEAD
git commit -am "Pin grammar to <rev>"
```

The public repository only has pushed commits. To try unpushed grammar changes with
Install Dev Extension, pin with `--local` (that checkout, via `file://`) and don't commit
that pin.
