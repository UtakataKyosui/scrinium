mod cli;
mod tui;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use cli::{Cli, Commands};
use scrinium::{bundle, document, graph, validate};
use std::{fs, path::PathBuf};
use uuid::Uuid;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Create {
            doc_type,
            title,
            output,
        } => cmd_create(&doc_type, &title, output),
        Commands::Validate { path } => cmd_validate(&path.unwrap_or_else(|| PathBuf::from("."))),
        Commands::Bundle { dir } => cmd_bundle(&dir.unwrap_or_else(|| PathBuf::from("."))),
        Commands::Graph {
            dir,
            format,
            output,
        } => cmd_graph(&dir.unwrap_or_else(|| PathBuf::from(".")), &format, output),
        Commands::Edit { path } => tui::run_editor(path),
    }
}

fn cmd_create(doc_type: &str, title: &str, output: Option<PathBuf>) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let slug = title.to_lowercase().replace(' ', "-");
    let path = output.unwrap_or_else(|| PathBuf::from(format!("{slug}.md")));

    let content = format!(
        "---\nid: \"{id}\"\ntype: \"{doc_type}\"\ntitle: \"{title}\"\ndescription: \"\"\nresource: \"\"\ntags: []\ntimestamp: \"{ts}\"\n---\n\n# {title}\n\n## Overview\n\n## Examples\n\n## Citations\n"
    );

    fs::write(&path, &content)?;
    println!("Created: {} (id: {})", path.display(), id);
    Ok(())
}

fn cmd_validate(path: &std::path::Path) -> Result<()> {
    let results = validate::validate_path(path)?;
    if results.is_empty() {
        println!("No Markdown files found in {}", path.display());
    } else {
        validate::print_results(&results);
    }
    Ok(())
}

fn cmd_bundle(dir: &std::path::Path) -> Result<()> {
    let b = bundle::Bundle::load(dir)?;
    let docs: Vec<_> = b.concept_docs().collect();
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // index.md
    let mut rows = String::new();
    for doc in &docs {
        let title = doc.frontmatter.title.as_deref().unwrap_or("(no title)");
        let rel = doc.path.strip_prefix(dir).unwrap_or(&doc.path);
        let rel_str = rel.to_string_lossy();
        let desc = doc.frontmatter.description.as_deref().unwrap_or("");
        rows.push_str(&format!(
            "| [{}](./{}) | {} | {} |\n",
            title, rel_str, doc.frontmatter.doc_type, desc
        ));
    }
    let index = format!(
        "---\ntype: \"Index\"\ntitle: \"Knowledge Bundle\"\ndescription: \"バンドルのドキュメント一覧\"\ntimestamp: \"{ts}\"\n---\n\n# Knowledge Bundle\n\n## Documents\n\n| Title | Type | Description |\n|-------|------|-------------|\n{rows}"
    );
    let index_path = dir.join("index.md");
    fs::write(&index_path, index)?;
    println!("Updated: {}", index_path.display());

    // log.md
    let entry = format!(
        "## {ts}\n\n- バンドルを更新しました ({} ドキュメント)\n\n",
        docs.len()
    );
    let log_path = dir.join("log.md");
    if log_path.exists() {
        let existing = fs::read_to_string(&log_path)?;
        let insert_at = document::frontmatter_end(&existing).unwrap_or(0);
        let head = existing[..insert_at].trim_end();
        let tail = existing[insert_at..].trim_start();
        fs::write(&log_path, format!("{head}\n\n{entry}{tail}"))?;
    } else {
        fs::write(
            &log_path,
            format!(
                "---\ntype: \"ChangeLog\"\ntitle: \"Change Log\"\ndescription: \"このバンドルの変更履歴\"\ntimestamp: \"{ts}\"\n---\n\n{entry}"
            ),
        )?;
    }
    println!("Updated: {}", log_path.display());

    Ok(())
}

fn cmd_graph(dir: &std::path::Path, format: &str, output: Option<PathBuf>) -> Result<()> {
    let b = bundle::Bundle::load(dir)?;
    let g = graph::GraphData::from_bundle(&b);

    let out = output.unwrap_or_else(|| dir.join(format!("graph.{format}")));

    match format {
        "json" => fs::write(&out, g.to_json()?)?,
        "yaml" => fs::write(&out, g.to_yaml()?)?,
        "svg" => fs::write(&out, g.to_svg())?,
        "png" => g.to_png(&out)?,
        other => anyhow::bail!("Unknown format '{other}'. Use: json, yaml, svg, png"),
    }

    println!(
        "Graph exported: {} ({} nodes, {} edges)",
        out.display(),
        g.nodes.len(),
        g.edges.len()
    );
    Ok(())
}
