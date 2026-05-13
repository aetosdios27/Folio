use crate::fetch::Paper;
use anyhow::{Context as AnyhowContext, Result};
use chrono::Local;
use std::fs;
use tera::{Context, Tera};

const DEFAULT_TEMPLATE: &str = r#"---
title: {{ title | json_encode() | safe }}
authors: {{ authors | json_encode() | safe }}
author_short: {{ author_short | json_encode() | safe }}
year: "{{ year }}"
published: "{{ published }}"
source: "{{ source }}"
topic: "{{ topic }}"
arxiv_id: "{{ arxiv_id }}"
url: "{{ url }}"
pdf: "{{ pdf_url }}"
status: unread
tags:
  - paper
  - "{{ topic_tag }}"
saved: "{{ saved_date }}"
---

# {{ title }}

> [!info] Meta
> **Authors:** {{ author_short }}
> **Published:** {{ published }}
> **Source:** [Abstract]({{ url }}){% if pdf_url %} · [PDF]({{ pdf_url }}){% endif %}

## Abstract

{{ abstract_text }}

## Reading Notes

> [!note] Your notes here

"#;

/// Build and return a Tera instance with the "note" template loaded.
/// Reads the user's template file from the config directory, falling back to
/// the built-in default if the file doesn't exist.
/// Called once at startup so subsequent renders hit only in-memory state.
pub fn build_tera() -> Result<Tera> {
    let mut tera = Tera::default();

    let config_dir = crate::config::get_project_dirs()?
        .config_dir()
        .to_path_buf();
    let template_path = config_dir.join("templates/note.md");

    let template_content = if template_path.exists() {
        fs::read_to_string(&template_path)?
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    tera.add_raw_template("note", &template_content)?;

    Ok(tera)
}

/// Render a single paper note using the pre-built (cached) Tera instance.
pub fn render_note(tera: &Tera, paper: &Paper, topic: &str) -> Result<String> {
    let mut context = Context::new();

    context.insert("title", &paper.title);
    context.insert("authors", &paper.authors);
    context.insert("author_short", &author_short(&paper.authors));
    context.insert("published", &paper.published);
    context.insert("year", &paper.published.chars().take(4).collect::<String>());
    context.insert("source", &paper.source);
    context.insert("topic", topic);
    // topic_tag: spaces → hyphens, lowercased, so Obsidian accepts it as a valid tag
    let topic_tag = topic.to_lowercase().replace(' ', "-");
    context.insert("topic_tag", &topic_tag);
    context.insert("arxiv_id", &paper.arxiv_id.as_deref().unwrap_or(""));
    context.insert("url", &paper.abs_url);
    context.insert("pdf_url", &paper.pdf_url.as_deref().unwrap_or(""));
    context.insert("abstract_text", &paper.abstract_text);
    context.insert("saved_date", &Local::now().format("%Y-%m-%d").to_string());

    let rendered = tera
        .render("note", &context)
        .context("Failed to render note. Check your ~/.config/folio/templates/note.md syntax.")?;

    Ok(rendered)
}

fn author_short(authors: &[String]) -> String {
    match authors {
        [] => "Unknown authors".into(),
        [one] => one.clone(),
        [first, second] => format!("{first}, {second}"),
        [first, second, ..] => format!("{first}, {second} et al."),
    }
}
