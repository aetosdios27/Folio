# Folio

![Crates.io](https://img.shields.io/crates/v/folio)
![Rust Edition](https://img.shields.io/badge/edition-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)

**A terminal UI for searching ArXiv and importing papers directly into your Obsidian vault as formatted markdown notes.**

---

## Features

- **Instant ArXiv search** — no API key, no rate limits, results in seconds
- **TUI-first** — browse, preview, and select papers without leaving your terminal
- **One-command import** — selected papers land in your vault as markdown notes with YAML frontmatter containing:
  - title
  - authors
  - abstract
  - publication date
  - ArXiv URL
- **Paginated results** — navigate through pages of results without retyping your query
- **Interactive onboarding** — set your vault path once on first launch; change it anytime with `--reconfigure`
- **Direct query launch** — skip the start screen and jump straight to results with `--query`

---

## Installation

Folio requires an [Obsidian vault](https://obsidian.md?utm_source=chatgpt.com).

### Option A — Install via Crates.io (Recommended)

Requires the [Rust toolchain](https://rustup.rs?utm_source=chatgpt.com).

```bash
cargo install folio
```

---

### Option B — Pre-built Binary (No Rust Needed)

Download the latest binary for your platform from the GitHub Releases page, extract it, and move it somewhere on your `$PATH`.

#### Example — Linux x86_64

```bash
curl -L https://github.com/aetos-dev/folio/releases/latest/download/folio-v0.2.0-x86_64-unknown-linux-gnu.tar.gz \
  | tar xz

sudo mv folio /usr/local/bin/
```

---

## Usage

```bash
folio                         # Launch the TUI
folio --query "your search"  # Launch directly into results
folio --reconfigure          # Re-run onboarding
folio --help                 # Show help
folio --version              # Show version
```

On first launch, Folio walks you through a short interactive setup to select:

- your Obsidian vault
- your papers folder

After setup, simply run:

```bash
folio
```

---

## Keybindings

| Key | Action |
|---|---|
| `j / k` or `↑ / ↓` | Move up / down |
| `J / K` | Scroll preview |
| `e` | Edit search query |
| `Enter` | Fetch papers / Import selected |
| `Space` | Toggle paper selection |
| `p` | Preview selected paper |
| `q` or `Ctrl+C` | Quit |

---

## Configuration

Configuration is stored at:

```text
~/.config/folio/config.toml
```

You can edit this file directly to change:

- vault path
- papers subfolder
- default behavior

To re-run the interactive onboarding instead:

```bash
folio --reconfigure
```

---

## License

MIT
