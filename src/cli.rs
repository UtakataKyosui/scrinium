use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "archivum", about = "OKF (Open Knowledge Format) CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new OKF document with a UUID
    Create {
        /// Document type (e.g., Concept, Playbook, API)
        #[arg(value_name = "TYPE")]
        doc_type: String,
        /// Document title
        title: String,
        /// Output file path (defaults to <slugified-title>.md)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate OKF compliance for a file or directory
    Validate {
        /// Path to file or directory (defaults to current directory)
        path: Option<PathBuf>,
    },
    /// Generate or update index.md and log.md for a bundle
    Bundle {
        /// Bundle directory (defaults to current directory)
        dir: Option<PathBuf>,
    },
    /// Export the knowledge graph
    Graph {
        /// Bundle directory (defaults to current directory)
        dir: Option<PathBuf>,
        /// Output format: json, yaml, svg, png
        #[arg(short, long, default_value = "svg")]
        format: String,
        /// Output file (defaults to graph.<format>)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Open the TUI editor
    Edit {
        /// File to open directly (omit to start in the file browser)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}
