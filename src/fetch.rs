use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ── Paper ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub published: String,
    pub abstract_text: String,
    pub source: String,
    pub citations: u64,
    pub venue: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub semantic_scholar_id: Option<String>,
    pub abs_url: String,
    pub pdf_url: Option<String>,
    pub tldr: Option<String>,
}

// ── ArXiv API ─────────────────────────────────────────────────────────────────

const ARXIV_BASE: &str = "http://export.arxiv.org/api/query";
const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

/// Fetch up to 20 papers from the ArXiv search API starting at `offset`.
pub async fn fetch_papers(query: &str, offset: usize) -> Result<Vec<Paper>> {
    let encoded_query = urlencoding::encode(query).into_owned();
    let url =
        format!("{ARXIV_BASE}?search_query=all:{encoded_query}&start={offset}&max_results=20");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")?;
    let xml_text = client
        .get(&url)
        .send()
        .await
        .context("Failed to reach ArXiv API")?
        .error_for_status()
        .context("ArXiv API returned an error status")?
        .text()
        .await
        .context("Failed to read ArXiv response body")?;

    parse_atom(&xml_text)
}

// ── XML parsing ───────────────────────────────────────────────────────────────

fn parse_atom(xml: &str) -> Result<Vec<Paper>> {
    let doc = roxmltree::Document::parse(xml).context("Failed to parse ArXiv Atom XML")?;
    let root = doc.root_element();

    let mut papers = Vec::new();

    for entry in root.children().filter(|n| {
        n.is_element()
            && n.tag_name().name() == "entry"
            && n.tag_name().namespace() == Some(ATOM_NS)
    }) {
        // <id> — full URL like http://arxiv.org/abs/2301.00001v1
        let id_url = child_text(&entry, "id", ATOM_NS).unwrap_or_default();

        // Extract the bare ArXiv ID from the URL path, stripping any version suffix.
        // e.g. "http://arxiv.org/abs/2301.00001v1" → "2301.00001"
        let arxiv_id: Option<String> = id_url
            .rsplit('/')
            .next()
            .map(|s| {
                // Remove version suffix (e.g. "v1", "v2")
                if let Some(pos) = s.rfind('v') {
                    let (base, ver) = s.split_at(pos);
                    // Only strip if everything after 'v' is digits
                    if ver[1..].chars().all(|c| c.is_ascii_digit()) {
                        base.to_string()
                    } else {
                        s.to_string()
                    }
                } else {
                    s.to_string()
                }
            })
            .filter(|s| !s.is_empty());

        let abs_url = arxiv_id
            .as_deref()
            .map(|id| format!("http://arxiv.org/abs/{id}"))
            .unwrap_or_else(|| id_url.clone());

        let pdf_url = arxiv_id
            .as_deref()
            .map(|id| format!("http://arxiv.org/pdf/{id}"));

        // <title> — collapse internal whitespace
        let title = child_text(&entry, "title", ATOM_NS)
            .map(normalize_whitespace)
            .unwrap_or_else(|| "Untitled".into());

        // <summary> — the abstract
        let abstract_text = child_text(&entry, "summary", ATOM_NS)
            .map(normalize_whitespace)
            .unwrap_or_default();

        // <published> — take the first 4 chars (the year)
        let published = child_text(&entry, "published", ATOM_NS)
            .and_then(|s| {
                let year: String = s.chars().take(4).collect();
                if year.len() == 4 {
                    Some(year)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown".into());

        // <author><name> — collect all authors
        let authors: Vec<String> = entry
            .children()
            .filter(|n| {
                n.is_element()
                    && n.tag_name().name() == "author"
                    && n.tag_name().namespace() == Some(ATOM_NS)
            })
            .filter_map(|author_node| child_text(&author_node, "name", ATOM_NS))
            .map(|s| normalize_whitespace(s))
            .collect();

        papers.push(Paper {
            id: id_url,
            title,
            authors,
            published,
            abstract_text,
            source: "ArXiv".into(),
            citations: 0,
            venue: None,
            doi: None,
            arxiv_id,
            semantic_scholar_id: None,
            abs_url,
            pdf_url,
            tldr: None,
        });
    }

    Ok(papers)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the trimmed text content of the first child element with the given
/// local name and namespace, if it exists.
fn child_text<'a>(node: &roxmltree::Node<'a, '_>, local: &str, ns: &str) -> Option<String> {
    node.children()
        .find(|n| {
            n.is_element() && n.tag_name().name() == local && n.tag_name().namespace() == Some(ns)
        })
        .map(|n| n.text().unwrap_or("").to_string())
}

/// Collapse runs of ASCII whitespace (spaces, newlines, tabs) into a single
/// space and trim the result.
fn normalize_whitespace(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
