mod cli;
mod output;

use clap::Parser;
use globset::Glob;
use fsx::collect;

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
            let ignore_glob = match &ignore {
                Some(pattern) => Some(match Glob::new(pattern) {
                    Ok(glob) => glob.compile_matcher(),
                    Err(e) => {
                        eprintln!("Invalid ignore pattern '{}': {}", pattern, e);
                        std::process::exit(1);
                    }
                }),
                None => None,
            };

            let report = collect(&path, max_depth, follow_symlinks, ignore_glob);
            {
                output::print_stats(&report.stats, format);
                for err in report.errors {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
