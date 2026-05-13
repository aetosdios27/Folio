# Folio

![Rust Edition](https://img.shields.io/badge/edition-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)

**A terminal UI for searching ArXiv and importing papers directly into your Obsidian vault as formatted markdown notes.**

---

## Features

- **Instant ArXiv search** — no API key, no rate limits, results in seconds
- **TUI-first** — browse, preview, and select papers without leaving your terminal
- **One-command import** — selected papers land in your vault as markdown notes with YAML frontmatter (title, authors, abstract, date, URL)
- **Paginated results** — navigate through pages of results without retyping your query
- **Interactive onboarding** — set your vault path once on first launch; change it anytime with `--reconfigure`
- **Direct query launch** — skip the start screen and jump straight to results with `--query`

---

## Installation

Requires the [Rust toolchain](https://rustup.rs) and an [Obsidian](https://obsidian.md) vault.

```sh
cargo install --git https://github.com/aetos-dev/folio
```

---

## Usage

```sh
folio                        # Launch the TUI
folio --query "your search"  # Launch directly into results
folio --reconfigure          # Re-run onboarding to change vault/papers folder
folio --help                 # Show help
folio --version              # Show version
```

On first launch, Folio walks you through a short setup to select your vault and papers folder. After that, just run `folio`.

---

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` or `↑` / `↓` | Move up / down |
| `J` / `K` | Scroll preview |
| `/` | Edit search query |
| `Enter` | Fetch papers (start screen) / Import selected (results screen) |
| `Space` | Toggle select paper |
| `n` / `p` | Next / previous page |
| `r` | Refresh / re-fetch |
| `b` | Go back |
| `q` or `Ctrl+C` | Quit |

---

## Configuration

Config is stored at `~/.config/folio/config.toml`. You can edit it directly to change your vault path or papers subfolder.

To re-run the interactive setup instead:

```sh
folio --reconfigure
```

---

## License

MIT
