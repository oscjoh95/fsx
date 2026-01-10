use crate::fsx;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Human,
    Raw,
    Debug,
}

fn convert_to_human_readable(bytes: u64) -> String {
    let prefixes: [&str; 5] = ["", "k", "M", "G", "T"];
    let mut prefix = 0;
    let mut new_val = bytes;
    while new_val > 1000 && prefix + 1 < prefixes.len() {
        new_val /= 1000;
        prefix += 1;
    }
    new_val.to_string() + prefixes[prefix] + "B"
}

pub fn print_stats(stats: &fsx::FsStats, format: OutputFormat) {
    match format {
        OutputFormat::Raw => {
            println!("Files: {}", stats.total_files);
            println!("Dirs: {}", stats.total_dirs);
            println!("Size: {} bytes", stats.total_size);
            if let Some((path, size)) = &stats.largest_file {
                println!("Largest file: {} ({} bytes)", path.display(), size);
            }
            println!("Max depth: {}", stats.max_depth);
        }
        OutputFormat::Debug => {
            println!("{:?}", stats);
        }
        OutputFormat::Human => {
            println!("Files: {}", stats.total_files);
            println!("Dirs: {}", stats.total_dirs);
            println!("Size: {}", convert_to_human_readable(stats.total_size));
            if let Some((path, size)) = &stats.largest_file {
                println!(
                    "Largest file: {} ({})",
                    path.display(),
                    convert_to_human_readable(*size)
                );
            }
            println!("Max depth: {}", stats.max_depth);
        }
    }
}
