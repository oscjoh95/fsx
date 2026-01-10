mod cli;
mod error;
mod fsx;
mod output;
mod walk;
mod test_utils;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Stats {
            path,
            max_depth,
            format,
        } => match fsx::collect_stats(&path, max_depth) {
            Ok(stats) => output::print_stats(&stats, format),
            Err(errs) => {
                for err in errs {
                    eprintln!("{}", err);
                }
            }
        },
    }
}
