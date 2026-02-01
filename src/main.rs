mod cli;
mod output;

use clap::Parser;
use fsx::{collect, find};
use fsx::filter::GitIgnoreFilter;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Stats {
            path,
            max_depth,
            format,
            follow_symlinks,
            ignore,
        } => {
            let ignore_filter =
                GitIgnoreFilter::from_gitignore(&path, &ignore.unwrap_or(Vec::new()));

            let report = collect(&path, max_depth, follow_symlinks, &ignore_filter);

            output::print_stats(&report.stats, format);
            for err in report.errors {
                eprintln!("{}", err);
            }
        }

        cli::Commands::Find {
            path,
            regex,
            max_depth,
            format,
            follow_symlinks,
            ignore,
        } => {
            let ignore_filter =
                GitIgnoreFilter::from_gitignore(&path, &ignore.unwrap_or(Vec::new()));

            let pattern = regex.unwrap_or(".*".to_string());

            let report = find(&path, max_depth, follow_symlinks, &ignore_filter, &pattern);
            
            output::print_find_entries(&report.entries, format);
            for err in report.errors {
                eprintln!("{}", err);
            }
        }
    }
}
