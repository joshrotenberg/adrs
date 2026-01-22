//! Generate documentation commands.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Generate a table of contents.
pub fn generate_toc(
    root: &Path,
    ordered: bool,
    intro: Option<PathBuf>,
    outro: Option<PathBuf>,
    prefix: Option<String>,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;
    let adrs = repo.list()?;

    // Print intro if provided
    if let Some(intro_path) = intro {
        let content = fs::read_to_string(&intro_path)
            .with_context(|| format!("Failed to read intro file: {}", intro_path.display()))?;
        println!("{}", content);
    }

    // Print TOC
    let prefix = prefix.unwrap_or_default();
    for (i, adr) in adrs.iter().enumerate() {
        let bullet = if ordered {
            format!("{}.", i + 1)
        } else {
            "*".to_string()
        };
        println!(
            "{} [{}]({}{})",
            bullet,
            adr.full_title(),
            prefix,
            adr.filename()
        );
    }

    // Print outro if provided
    if let Some(outro_path) = outro {
        let content = fs::read_to_string(&outro_path)
            .with_context(|| format!("Failed to read outro file: {}", outro_path.display()))?;
        println!("{}", content);
    }

    Ok(())
}

/// Generate a Graphviz graph.
pub fn generate_graph(root: &Path, prefix: Option<String>, extension: &str) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;
    let adrs = repo.list()?;

    let prefix = prefix.unwrap_or_default();

    println!("digraph {{");
    println!("  node [shape=plaintext];");

    // Create nodes
    for adr in &adrs {
        let filename = adr.filename().replace(".md", &format!(".{}", extension));
        println!(
            "  _{} [label=\"{}\"; URL=\"{}{}\"];",
            adr.number,
            adr.full_title(),
            prefix,
            filename
        );
    }

    // Create sequential edges (dotted)
    println!("  edge [style=dotted, weight=10];");
    for window in adrs.windows(2) {
        println!("  _{} -> _{};", window[0].number, window[1].number);
    }

    // Create relationship edges
    println!("  edge [style=solid, weight=1];");
    for adr in &adrs {
        for link in &adr.links {
            println!(
                "  _{} -> _{} [label=\"{}\"];",
                adr.number, link.target, link.kind
            );
        }
    }

    println!("}}");

    Ok(())
}

/// Generate an mdbook.
pub fn generate_book(
    root: &Path,
    output: &Path,
    title: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;
    let adrs = repo.list()?;

    let title = title.unwrap_or_else(|| "Architecture Decision Records".to_string());
    let description =
        description.unwrap_or_else(|| "Documentation of architectural decisions".to_string());
    let author = whoami::username();

    // Create output directories
    let src_dir = output.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create book.toml
    let book_toml = format!(
        r#"[book]
title = "{}"
description = "{}"
authors = ["{}"]
language = "en"

[build]
build-dir = "book"
"#,
        title, description, author
    );
    fs::write(output.join("book.toml"), book_toml)?;

    // Create SUMMARY.md
    let mut summary = String::from("# Summary\n\n");
    for adr in &adrs {
        summary.push_str(&format!("- [{}]({})\n", adr.full_title(), adr.filename()));

        // Copy ADR file to src
        if let Some(path) = &adr.path {
            let content = fs::read_to_string(path)?;
            fs::write(src_dir.join(adr.filename()), content)?;
        }
    }
    fs::write(src_dir.join("SUMMARY.md"), summary)?;

    println!("{}", output.display());
    Ok(())
}
