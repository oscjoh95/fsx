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
        /// Output format
        ///
        /// Possible values:
        ///
        /// human  - Human-readable sizes
        ///
        /// raw    - Exact byte counts
        ///
        /// debug  - Debug output (Rust struct dump)
        #[arg(long, default_value = "human")]
        format: output::OutputFormat,
    },
}
