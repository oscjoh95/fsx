mod cli;
mod output;

use clap::Parser;
use fsx::collect;
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
            let ignore_filter = GitIgnoreFilter::new(&path, &ignore.unwrap_or(Vec::new()));

            let report = collect(&path, max_depth, follow_symlinks, &ignore_filter);
            {
                output::print_stats(&report.stats, format);
                for err in report.errors {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
