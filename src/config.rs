use anyhow::{Context, Result};
use directories::ProjectDirs;
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

// --- Interactive Directory Browser ---
// Presents the contents of `current` as a Select list.
// `prompt`      — the question shown above the list.
// `allow_new`   — when true, surfaces a "+ Create new folder here" option
//                 so the user can mint a folder that doesn't exist yet.
fn browse_for_directory(start: &Path, prompt: &str, allow_new: bool) -> Result<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let mut entries: Vec<PathBuf> = fs::read_dir(&current)
            .with_context(|| format!("Cannot read directory: {}", current.display()))?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        entries.sort();

        let mut options: Vec<String> = Vec::new();
        options.push(format!("✅  Select: {}/", current.display()));
        if allow_new {
            options.push("➕  Create a new folder here".to_string());
        }
        if current.parent().is_some() {
            options.push("📁  ../  (go up)".to_string());
        }
        for path in &entries {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            options.push(format!("📂  {name}/"));
        }

        let display = format!("{prompt}  [{}]", current.display());
        let choice = Select::new(&display, options)
            .with_help_message("↑↓ navigate · Enter to open · ✅ to confirm")
            .prompt()?;

        if choice.starts_with("✅") {
            return Ok(current);
        } else if choice.starts_with("➕") {
            let name = Text::new("New folder name:")
                .with_initial_value("Papers")
                .prompt()?;
            let new_dir = current.join(name.trim());
            fs::create_dir_all(&new_dir)?;
            return Ok(new_dir);
        } else if choice.starts_with("📁") {
            current = current.parent().unwrap_or(&current).to_path_buf();
        } else {
            let name = choice.trim_start_matches("📂  ").trim_end_matches('/');
            current = current.join(name);
        }
    }
}
// --------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topic {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub obsidian_vault: PathBuf,
    pub papers_dir: String,
    pub semantic_scholar_api_key: Option<String>,
    pub topics: Vec<Topic>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            obsidian_vault: PathBuf::new(),
            papers_dir: "Papers".to_string(),
            semantic_scholar_api_key: None,
            topics: vec![
                Topic {
                    name: "Database Storage Engines".into(),
                    query: "LSM tree B-tree WAL compaction storage engine".into(),
                },
                Topic {
                    name: "Database Transactions".into(),
                    query: "MVCC transactions OCC 2PL serializability".into(),
                },
                Topic {
                    name: "Database Query Optimization".into(),
                    query: "query optimizer cardinality estimation join ordering".into(),
                },
                Topic {
                    name: "Database OLAP & Columnar".into(),
                    query: "columnar execution vectorized processing OLAP parquet".into(),
                },
                Topic {
                    name: "Database Time-Series".into(),
                    query: "time series database compression retention downsampling".into(),
                },
                Topic {
                    name: "Database Streaming".into(),
                    query: "stream processing event streaming materialized views".into(),
                },
                Topic {
                    name: "Database Distributed".into(),
                    query: "distributed SQL replication sharding consensus".into(),
                },
                Topic {
                    name: "Database NewSQL".into(),
                    query: "newsql cockroachdb spanner distributed transactions".into(),
                },
                Topic {
                    name: "Distributed Consensus".into(),
                    query: "raft paxos byzantine consensus distributed systems".into(),
                },
                Topic {
                    name: "Distributed Replication".into(),
                    query: "leader follower replication quorum anti entropy".into(),
                },
                Topic {
                    name: "Distributed Scheduling".into(),
                    query: "distributed scheduler resource allocation cluster orchestration".into(),
                },
                Topic {
                    name: "Distributed Failure Detection".into(),
                    query: "failure detector gossip heartbeat distributed systems".into(),
                },
                Topic {
                    name: "Distributed Service Meshes".into(),
                    query: "service mesh envoy istio traffic management".into(),
                },
                Topic {
                    name: "Distributed Coordination".into(),
                    query: "zookeeper etcd coordination distributed locking".into(),
                },
                Topic {
                    name: "Distributed Event-Driven".into(),
                    query: "event driven architecture pubsub message queues kafka".into(),
                },
                Topic {
                    name: "Distributed Stream Processing".into(),
                    query: "flink beam stream processing exactly once".into(),
                },
                Topic {
                    name: "Networking Transport".into(),
                    query: "TCP QUIC congestion control networking".into(),
                },
                Topic {
                    name: "Networking Application".into(),
                    query: "HTTP RPC gRPC protocol design".into(),
                },
                Topic {
                    name: "Networking Datacenter".into(),
                    query: "datacenter networking RDMA leaf spine topology".into(),
                },
                Topic {
                    name: "Networking Load Balancing".into(),
                    query: "load balancing consistent hashing traffic routing".into(),
                },
                Topic {
                    name: "Networking CDNs & Edge".into(),
                    query: "content delivery network edge caching".into(),
                },
                Topic {
                    name: "Networking Virtualization".into(),
                    query: "SDN NFV programmable networking".into(),
                },
                Topic {
                    name: "OS Kernel Internals".into(),
                    query: "kernel scheduler interrupts syscall internals".into(),
                },
                Topic {
                    name: "OS Memory Management".into(),
                    query: "virtual memory paging allocator NUMA".into(),
                },
                Topic {
                    name: "OS Filesystems".into(),
                    query: "filesystem journaling ext4 btrfs zfs".into(),
                },
                Topic {
                    name: "OS IO Systems".into(),
                    query: "io_uring async IO DMA polling".into(),
                },
                Topic {
                    name: "OS Containers & Isolation".into(),
                    query: "cgroups namespaces container isolation".into(),
                },
                Topic {
                    name: "OS Virtualization".into(),
                    query: "hypervisor KVM virtualization".into(),
                },
                Topic {
                    name: "Performance Caching".into(),
                    query: "cache eviction cache coherence distributed cache".into(),
                },
                Topic {
                    name: "Performance Memory".into(),
                    query: "NUMA cache locality memory hierarchy".into(),
                },
                Topic {
                    name: "Performance Engineering".into(),
                    query: "latency throughput tail latency profiling".into(),
                },
                Topic {
                    name: "Performance Observability".into(),
                    query: "distributed tracing observability profiling telemetry".into(),
                },
                Topic {
                    name: "Compilers Optimizations".into(),
                    query: "SSA compiler optimization IR inlining".into(),
                },
                Topic {
                    name: "Compilers Runtimes".into(),
                    query: "JVM garbage collection runtime systems".into(),
                },
                Topic {
                    name: "Compilers Garbage Collection".into(),
                    query: "garbage collector concurrent GC memory reclamation".into(),
                },
                Topic {
                    name: "Compilers JIT Compilation".into(),
                    query: "JIT runtime optimization speculative execution".into(),
                },
                Topic {
                    name: "Cloud Scheduling".into(),
                    query: "kubernetes scheduler autoscaling orchestration".into(),
                },
                Topic {
                    name: "Cloud Serverless".into(),
                    query: "serverless FaaS cold starts lambda".into(),
                },
                Topic {
                    name: "Cloud Infrastructure".into(),
                    query: "infrastructure as code terraform orchestration".into(),
                },
                Topic {
                    name: "Cloud Reliability (SRE)".into(),
                    query: "SRE reliability fault tolerance chaos engineering".into(),
                },
                Topic {
                    name: "Security Authentication".into(),
                    query: "authentication OAuth OIDC SSO identity".into(),
                },
                Topic {
                    name: "Security Distributed".into(),
                    query: "zero trust distributed auth secure systems".into(),
                },
                Topic {
                    name: "Security Cryptography".into(),
                    query: "cryptography distributed cryptographic protocols".into(),
                },
                Topic {
                    name: "Search Engines".into(),
                    query: "inverted index retrieval ranking BM25".into(),
                },
                Topic {
                    name: "Search Vector Databases".into(),
                    query: "ANN HNSW vector similarity search".into(),
                },
                Topic {
                    name: "Search Information Retrieval".into(),
                    query: "retrieval systems ranking relevance".into(),
                },
                Topic {
                    name: "AI Inference Systems".into(),
                    query: "LLM inference batching KV cache serving".into(),
                },
                Topic {
                    name: "AI GPU Systems".into(),
                    query: "GPU scheduling CUDA memory systems".into(),
                },
                Topic {
                    name: "AI Distributed Training".into(),
                    query: "distributed deep learning allreduce training systems".into(),
                },
                Topic {
                    name: "AI Model Serving".into(),
                    query: "model serving inference infrastructure".into(),
                },
            ],
        }
    }
}

pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "aetos", "folio")
        .context("Could not determine the home directory for your operating system")
}

pub fn load_or_init_config() -> Result<Config> {
    let proj_dirs = get_project_dirs()?;
    let config_dir = proj_dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let config_str =
            fs::read_to_string(&config_path).context("Failed to read existing config.toml")?;
        let config: Config =
            toml::from_str(&config_str).context("Config file is corrupted or invalid TOML")?;
        Ok(config)
    } else {
        run_onboarding(config_dir, &config_path)
    }
}

fn print_banner() {
    println!();
    println!("  ╔══════════════════════════════════════╗");
    println!("  ║                                      ║");
    println!("  ║   📚  Welcome to Folio               ║");
    println!("  ║   Your Obsidian research companion   ║");
    println!("  ║                                      ║");
    println!("  ╚══════════════════════════════════════╝");
    println!();
    println!("  This is a one-time setup. We need to know");
    println!("  where your Obsidian vault lives so Folio");
    println!("  can save imported papers directly into it.");
    println!();
}

fn print_step(n: u8, total: u8, label: &str) {
    println!();
    println!("  ── Step {n}/{total}: {label} ─────────────────────");
    println!();
}

fn print_success(label: &str, value: &str) {
    println!("  ✅  {label}");
    println!("      {value}");
}

fn run_onboarding(config_dir: &std::path::Path, config_path: &std::path::Path) -> Result<Config> {
    print_banner();

    // ── Step 1: Vault ────────────────────────────────────────────────────────
    print_step(1, 2, "Locate your Obsidian Vault");
    println!("  Browse to the root folder of your vault.");
    println!("  Use ↑↓ to move, Enter to open a folder,");
    println!("  then choose ✅ when you're in the right place.");
    println!();

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let vault_path = browse_for_directory(&home, "Vault location", false)?;
    print_success("Vault set to", &format!("{}/", vault_path.display()));

    // ── Step 2: Papers folder ────────────────────────────────────────────────
    print_step(2, 2, "Choose your Papers folder");

    let papers_path = {
        let default_papers = vault_path.join("Papers");

        if default_papers.exists() {
            println!("  Found an existing Papers/ folder inside your vault.");
            println!("  Confirm it below, or navigate to a different one.");
            println!();
            browse_for_directory(&vault_path, "Papers folder", false)?
        } else {
            println!("  No Papers/ folder exists yet inside your vault.");
            println!();
            let use_default = Confirm::new("Create a folder called 'Papers' inside your vault?")
                .with_default(true)
                .with_help_message("Enter = yes · n = browse and pick a custom name")
                .prompt()?;

            if use_default {
                default_papers
            } else {
                println!();
                println!("  Browse to an existing folder, or use ➕ to create a new one.");
                println!();
                browse_for_directory(&vault_path, "Papers folder", true)?
            }
        }
    };

    let papers_dir = papers_path
        .strip_prefix(&vault_path)
        .unwrap_or(&papers_path)
        .to_string_lossy()
        .to_string();

    fs::create_dir_all(&papers_path)?;
    init_obsidian_workspace(&papers_path)?;
    print_success(
        "Papers folder set to",
        &format!("{}/", papers_path.display()),
    );

    // ── Save ─────────────────────────────────────────────────────────────────
    let config = Config {
        obsidian_vault: vault_path,
        papers_dir: papers_dir.trim_matches('/').to_string(),
        ..Config::default()
    };

    fs::create_dir_all(config_dir)?;
    init_config_templates(config_dir)?;

    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(config_path, toml_string)?;

    println!();
    println!("  ╔══════════════════════════════════════╗");
    println!("  ║   🎉  All set!                        ║");
    println!("  ╚══════════════════════════════════════╝");
    println!();
    println!("  Config saved to:");
    println!("  {}", config_path.display());
    println!();
    println!("  To change your vault or papers folder later,");
    println!("  edit that file directly or delete it to re-run setup.");
    println!();

    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(config)
}

fn init_obsidian_workspace(folder: &std::path::Path) -> Result<()> {
    let inbox_path = folder.join("inbox.md");
    let dashboard_path = folder.join("Papers Dashboard.md");

    let index_template = r#"---
title: Papers Hub
cssclasses: [dashboard]
---

# 📚 Papers Hub

> [!info] Welcome to your Folio Workspace
> Papers imported via Folio will automatically appear in the **Inbox** below.

## 📥 Inbox
*Review, tag, and file these newly imported papers:*

<!-- CLI appends new papers below this line -->
"#;

    let dataview_template = r#"# 📊 Papers Dashboard

## 📖 Unread Papers
```dataview
TABLE author_short as Authors, year as Year, topic as Topic
FROM "Papers"
WHERE status = "unread"
SORT year DESC
🧠 Reading by Topic
Code snippet
TABLE length(rows) as Count
FROM "Papers"
GROUP BY topic
"#;

    if !inbox_path.exists() {
        fs::write(inbox_path, index_template)?;
    }
    if !dashboard_path.exists() {
        fs::write(dashboard_path, dataview_template)?;
    }

    Ok(())
}

fn init_config_templates(config_dir: &std::path::Path) -> Result<()> {
    let templates_dir = config_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    let note_template_path = templates_dir.join("note.md");
    let default_note_template = r#"---
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
  - "{{ topic }}"
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

    // Always overwrite so fixes to this template are picked up on re-install.
    fs::write(note_template_path, default_note_template)?;

    Ok(())
}
