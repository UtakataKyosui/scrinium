# scrinium

[![Crates.io](https://img.shields.io/crates/v/scrinium)](https://crates.io/crates/scrinium)
[![CI](https://github.com/UtakataKyosui/scrinium/actions/workflows/ci.yml/badge.svg)](https://github.com/UtakataKyosui/scrinium/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A CLI and TUI editor for managing [Open Knowledge Format (OKF)](https://github.com/UtakataKyosui/scrinium) Markdown knowledge graphs.

OKF準拠のMarkdownドキュメントを作成・検証・バンドル・可視化するCLI/TUIツールです。

---

## Features

- **Create** — generate new OKF documents with UUID v4 and ISO 8601 timestamp automatically
- **Validate** — check OKF frontmatter compliance across a file or entire directory
- **Bundle** — auto-generate `index.md` (catalog) and `log.md` (changelog) for a knowledge directory
- **Graph** — export the knowledge graph (document nodes + `[[wikilink]]` edges) as JSON, YAML, SVG, or PNG
- **TUI editor** — three-panel terminal UI: file browser, frontmatter editor, Markdown editor with heading highlights and auto-save timestamp

---

## Installation

### From crates.io

```sh
cargo install scrinium
```

### From source

```sh
git clone https://github.com/UtakataKyosui/scrinium
cd scrinium
cargo install --path .
```

---

## Quick Start

```sh
# Create a new OKF document
scrinium create -T Concept --title "Rust Ownership"

# Validate all documents in the current directory
scrinium validate

# Open the TUI editor
scrinium edit
```

---

## CLI Reference

### `scrinium create`

Creates a new OKF Markdown document with a UUID and timestamp.

| Flag | Short | Description |
|---|---|---|
| `--type <TYPE>` | `-T` | Document type (e.g. `Concept`, `Playbook`, `API`) — **required** |
| `--title <TITLE>` | `-t` | Document title — **required** |
| `--output <PATH>` | `-o` | Output file path (default: `<slugified-title>.md`) |

```sh
scrinium create -T Concept --title "My Concept"
# → Creates my-concept.md with auto-generated id and timestamp
```

### `scrinium validate`

Validates OKF frontmatter compliance for a file or directory. Hidden directories (`.claude`, `.git`, etc.) are automatically excluded.

| Flag | Short | Description |
|---|---|---|
| `--path <PATH>` | `-p` | File or directory to validate (default: `.`) |

### `scrinium bundle`

Generates or updates `index.md` (document catalog) and `log.md` (activity log) for a knowledge bundle directory.

| Flag | Short | Description |
|---|---|---|
| `--dir <DIR>` | `-d` | Bundle directory (default: `.`) |

### `scrinium graph`

Exports the knowledge graph. Documents are nodes; `[[wikilink]]` references in the Markdown body become edges.

| Flag | Short | Description |
|---|---|---|
| `--dir <DIR>` | `-d` | Bundle directory (default: `.`) |
| `--format <FMT>` | `-f` | Output format: `json`, `yaml`, `svg`, `png` (default: `svg`) |
| `--output <PATH>` | `-o` | Output file (default: `graph.<format>`) |

```sh
scrinium graph --format png --output knowledge.png
```

### `scrinium edit`

Opens the TUI editor.

| Flag | Short | Description |
|---|---|---|
| `--file <FILE>` | `-f` | File to open directly (omit to start in the file browser) |

---

## OKF Document Format

Every OKF document begins with a YAML frontmatter block:

```yaml
---
id: "550e8400-e29b-41d4-a716-446655440000"   # UUID v4, auto-generated
type: "Concept"                               # required
title: "Rust Ownership"
description: "How Rust manages memory without GC"
resource: "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html"
tags: [rust, memory, ownership]
timestamp: "2026-01-01T00:00:00Z"            # ISO 8601, auto-updated on TUI save
---
```

| Field | Required | Description |
|---|---|---|
| `type` | Yes | Document classification |
| `id` | Recommended | UUID v4 identifier |
| `title` | Recommended | Human-readable title |
| `description` | Recommended | Short summary |
| `resource` | Optional | Reference URL |
| `tags` | Optional | YAML list of tags |
| `timestamp` | Optional | Auto-updated on TUI save |

Extra frontmatter fields are preserved and can be added freely from the TUI (Ctrl+N).

---

## TUI Key Bindings

### Global

| Key | Action |
|---|---|
| `Tab` | Cycle panel focus: Browse → Frontmatter → Markdown |
| `Shift+Tab` | Reverse cycle |
| `Ctrl+S` | Save file (auto-updates `timestamp`) |
| `Ctrl+Q` | Quit |
| `F1` | Toggle help popup |

### File Browser panel

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Open selected file |
| `Esc` | Quit |

### Frontmatter panel

| Key | Action |
|---|---|
| `↑` / `↓` | Move between fields |
| `Enter` | Edit selected field |
| `Ctrl+N` | Add a new custom YAML field |

### Markdown panel

Standard multi-line text editing via `tui-textarea`. Markdown headings (`#`, `##`, `###`) are syntax-highlighted.

---

## Contributing

Bug reports, feature requests, and pull requests are welcome.

1. Fork the repository
2. Create a branch: `git checkout -b feat/your-feature`
3. Commit using conventional commits: `feat(scrinium): add X`
4. Push and open a pull request

Before submitting, make sure the following pass:

```sh
cargo fmt
cargo clippy -- -D warnings
cargo test
```

---

## License

MIT — see [LICENSE](LICENSE) for details.
