use crate::filter::GitIgnoreFilter;
use std::path::Path;

pub enum FsNode<'a> {
    File(&'a str, &'a str),
    Dir(&'a str, Vec<FsNode<'a>>),
    #[cfg(target_os = "windows")]
    SymlinkFile(&'a str, &'a str), // (link name, target relative path)
    #[cfg(target_os = "windows")]
    SymlinkDir(&'a str, &'a str), // (link name, target relative path)
}

pub fn create_fs_tree(root: &Path, node: &FsNode) -> std::io::Result<()> {
    match node {
        FsNode::File(name, contents) => {
            std::fs::write(root.join(name), contents)?;
        }
        FsNode::Dir(name, children) => {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir)?;
            for child in children {
                create_fs_tree(&dir, child)?;
            }
        }
        #[cfg(target_os = "windows")]
        FsNode::SymlinkFile(name, target) => {
            let link_path = root.join(name);
            let target_path = root.join(target);
            std::os::windows::fs::symlink_file(target_path, link_path)?;
        }
        #[cfg(target_os = "windows")]
        FsNode::SymlinkDir(name, target) => {
            let link_path = root.join(name);
            let target_path = root.join(target);
            std::os::windows::fs::symlink_dir(target_path, link_path)?;
        }
    }
    Ok(())
}

pub fn gitignore_filter(root: &Path, patterns: &[&str]) -> GitIgnoreFilter {
    let patterns: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();
    GitIgnoreFilter::new(root, &patterns)
}

// Compares two floats with default tolerance
const EPS: f32 = 1e-6;
pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

// Compares two floats with tolerance
pub fn approx_eq_eps(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}
