use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::output;

#[derive(Parser)]
#[command(name = "fsx", version, about = "Filesystem exploration tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compute filesystem statistics for a directory tree
    Stats {
        /// root directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Limit recursion to a maximum depth
        ///
        /// Depth starts at 1 for entries directly under PATH
        ///
        /// If not set, the entire directory tree is traversed.
        #[arg(short, long)]
        max_depth: Option<usize>,
        /// Output format: human (default), raw (bytes), debug (Rust struct)
        #[arg(long, default_value = "human")]
        format: output::OutputFormat,
        /// Recurse into symbolic links
        #[arg(long)]
        follow_symlinks: bool,
        /// Ignore filter (gitignore semantics)
        #[arg(short, long)]
        ignore: Option<Vec<String>>,
    },

    /// Find matching nodes in the directory tree
    Find {
        /// root directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Regex pattern for file names
        #[arg(short, long)]
        regex: Option<String>,
        /// Limit recursion to a maximum depth
        ///
        /// Depth starts at 1 for entries directly under PATH
        ///
        /// If not set, the entire directory tree is traversed.
        #[arg(short, long)]
        max_depth: Option<usize>,
        /// Output format: human (default), raw (bytes), debug (Rust struct)
        #[arg(long, default_value = "human")]
        format: output::OutputFormat,
        /// Recurse into symbolic links
        #[arg(long)]
        follow_symlinks: bool,
        /// Ignore filter (gitignore semantics)
        #[arg(short, long)]
        ignore: Option<Vec<String>>,
    },
}
