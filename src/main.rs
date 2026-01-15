mod cli;
mod error;
mod fsx;
mod output;
#[cfg(test)]
mod test_utils;
mod walk;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Stats {
            path,
            max_depth,
            format,
            follow_symlinks
        } => {
            let report = fsx::collect_stats(&path, max_depth, follow_symlinks);
            {
                output::print_stats(&report.stats, format);
                for err in report.errors {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
