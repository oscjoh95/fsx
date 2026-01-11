mod cli;
mod error;
mod fsx;
mod output;
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
        } => {
            let report = fsx::collect_stats(&path, max_depth);
            {
                output::print_stats(&report.stats, format);
                for err in report.errors {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
