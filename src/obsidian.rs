use crate::app::ImportReport;
use crate::config::load_or_init_config;
use crate::fetch::Paper;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Import papers into the Obsidian vault.
/// `rendered` is a parallel slice of pre-rendered note content, one entry per paper,
/// produced by `templates::render_note` on the async side using the cached Tera engine.
pub fn import_to_obsidian(
    papers: &[Paper],
    topic: &str,
    rendered: &[String],
) -> Result<ImportReport> {
    let config = load_or_init_config()?;
    let folder = config.obsidian_vault.join(&config.papers_dir);

    // Ensure the destination exists
    fs::create_dir_all(&folder)?;

    let mut files = Vec::with_capacity(papers.len());
    let mut written = 0;
    let mut skipped = 0;

    for (i, paper) in papers.iter().enumerate() {
        let filename = note_filename(paper);
        let file_path = resolve_path(&folder, &filename, paper)?;

        // Atomic write: create_new fails if the file already exists, which
        // means this is a true re-import of the same paper — skip it.
        let file_write_attempt = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path);

        match file_write_attempt {
            Ok(mut file) => {
                file.write_all(rendered[i].as_bytes())?;
                written += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                skipped += 1;
            }
            Err(e) => return Err(e).context("Failed to write paper file"),
        }

        // Log to inbox
        append_to_inbox(&folder, paper, topic, &file_path)?;
        files.push(file_path);
    }

    Ok(ImportReport {
        written,
        skipped,
        folder,
        files,
    })
}

/// Append a paper link to inbox.md, grouped under a ## YYYY-MM-DD date heading.
/// Skips silently if the paper is already present (duplicate guard).
fn append_to_inbox(folder: &Path, paper: &Paper, topic: &str, file: &Path) -> Result<()> {
    let inbox_path = folder.join("inbox.md");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let date_heading = format!("\n## {}", today);

    let stem = file
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled");

    let paper_link = format!(
        "- [ ] [[{}|{}]] — {} `{}`",
        stem,
        paper.title.trim(),
        paper.published.trim(),
        topic.trim()
    );

    let mut content = fs::read_to_string(&inbox_path).unwrap_or_default();

    // 1. Duplicate guard
    if content.contains(&format!("[[{stem}|")) {
        return Ok(());
    }

    // 2. Insert today's date heading if it isn't already there
    if !content.contains(&date_heading) {
        content.push_str(&date_heading);
        content.push('\n');
    }

    // 3. Append the paper link under the heading
    content.push_str(&paper_link);
    content.push('\n');

    fs::write(inbox_path, content)?;
    Ok(())
}

/// Build the initial candidate filename stem for a paper.
/// Papers with a stable external ID (ArXiv, Semantic Scholar, DOI) embed that
/// ID in the stem, so they are inherently collision-free across imports.
/// Papers with no external ID rely on title + year alone, which can collide.
fn note_filename(paper: &Paper) -> String {
    let year = paper.published.chars().take(4).collect::<String>();
    let safe_title = slugify(&paper.title);

    let mut stem = format!("{year}-{safe_title}");

    if let Some(arxiv_id) = &paper.arxiv_id {
        stem.push('-');
        stem.push_str(&slugify(arxiv_id));
    } else if let Some(semantic_id) = &paper.semantic_scholar_id {
        stem.push('-');
        stem.push_str(&slugify(semantic_id));
    } else if let Some(doi) = &paper.doi {
        stem.push('-');
        stem.push_str(&slugify(doi));
    }

    // Truncate to avoid OS path-length limits (leave room for " N" suffix)
    if stem.chars().count() > 96 {
        stem = stem.chars().take(96).collect();
    }

    format!("{stem}.md")
}

/// Return the path to write this paper to, guaranteeing no unintended overwrite.
///
/// Strategy:
/// 1. Try the natural candidate path produced by `note_filename`.
/// 2. If that path is free, use it.
/// 3. If that path is occupied by the *same* paper (matching `paper.id`),
///    use it too — the subsequent `create_new` open will return `AlreadyExists`
///    and the caller will count it as a skip, which is correct.
/// 4. If that path is occupied by a *different* paper (title slug collision),
///    increment a counter suffix (Title-2.md, Title-3.md, …) until a free
///    slot is found.
fn resolve_path(folder: &Path, filename: &str, paper: &Paper) -> Result<PathBuf> {
    let candidate = folder.join(filename);

    if !candidate.exists() {
        return Ok(candidate);
    }

    // File exists — check whether it belongs to this paper or a different one.
    let existing = fs::read_to_string(&candidate).unwrap_or_default();
    if file_belongs_to_paper(&existing, paper) {
        // Same paper: let the atomic open handle the skip.
        return Ok(candidate);
    }

    // Different paper with the same slug — find a free numbered slot.
    let stem = filename.strip_suffix(".md").unwrap_or(filename);
    for n in 2u32..=999 {
        let numbered = folder.join(format!("{stem}-{n}.md"));
        if !numbered.exists() {
            return Ok(numbered);
        }
        // Slot is taken — check if *that* file is this paper before moving on.
        let existing = fs::read_to_string(&numbered).unwrap_or_default();
        if file_belongs_to_paper(&existing, paper) {
            return Ok(numbered);
        }
    }

    // Absolute fallback: timestamp suffix (should never be reached in practice).
    Ok(folder.join(format!("{stem}-{}.md", chrono::Local::now().timestamp())))
}

/// Heuristic: does an already-written note file belong to `paper`?
/// Checks for the Semantic Scholar ID in the frontmatter, which is always
/// present (we embed it for every paper regardless of other IDs).
fn file_belongs_to_paper(content: &str, paper: &Paper) -> bool {
    if let Some(ref id) = paper.semantic_scholar_id {
        return content.contains(id.as_str());
    }
    if let Some(ref arxiv) = paper.arxiv_id {
        return content.contains(arxiv.as_str());
    }
    if let Some(ref doi) = paper.doi {
        return content.contains(doi.as_str());
    }
    // No stable ID available — cannot confirm ownership, treat as different.
    false
}

// FIX: Unicode-friendly slug generator. Keeps letters, numbers, and dashes.
fn slugify(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
