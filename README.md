# project-tree

A simple ascii file tree generator. Designed to be used in project root. By default it will print to stdout, and copy to clipboard. By default it will not recurse into node_modules, .git, or .vscode folders.

```rust
//! TODO:
//! Make ignore / stop check more elegant, is HashMap<PathBuf> really the best way to do this?
```

## Usage

```bash
project-tree [flags] [options]
```

## Flags

| Flag | Description |
| --- | --- |
| --node_modules | Include node_modules |
| --git | Include .git |
| --vscode | Include .vscode |
| -r, --root | Include parent directory in tree, and indent all other files |
| -d, --dirs | Prioritize directories over files (default alphabetical) |

## Options

| Option | Arg | Description |
| --- | --- | --- |
| -o, --output | path | Output file |
| -i, --ignore | path | A file/folder to ignore, can be repeated |
| -s, --stop | path | A file/folder to not recurse into, can be repeated |

## Examples

```bash
project-tree -i Cargo.lock -s target -r
```

```bash
project-tree
├── src/
│   └── main.rs
├── target/
├── .gitignore
├── Cargo.toml
└── README.md
```
