use crate::error;
use crate::error::FsError;
use crate::walk;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct FsStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
    pub largest_file: Option<(PathBuf, u64)>,
    pub max_depth: usize,
}

pub fn collect_stats(
    root: &Path,
    max_depth: Option<usize>,
) -> Result<FsStats, Vec<error::FsError>> {
    let mut stats = FsStats::default();

    let mut entry_callback =
        |path: &Path, meta: &fs::Metadata, entry_type: walk::EntryType, depth: usize| {
            match entry_type {
                walk::EntryType::File => {
                    let size = meta.len();
                    stats.total_files += 1;
                    stats.total_size += size;
                    if stats
                        .largest_file
                        .as_ref()
                        .map_or(true, |(_, largest_size)| size > *largest_size)
                    {
                        stats.largest_file = Some((path.to_path_buf(), size));
                    }
                }
                walk::EntryType::Dir => stats.total_dirs += 1,
                walk::EntryType::Symlink => {} // Skip for now
            }
            stats.max_depth = stats.max_depth.max(depth);
        };

    let mut errs: Vec<FsError> = Vec::new();
    let mut error_callback = |err: FsError| {
        errs.push(err);
    };

    walk::walk_dir(root, &mut entry_callback, &mut error_callback, max_depth);
    if errs.is_empty() {
        Ok(stats)
    } else {
        Err(errs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FsNode, create_fs_tree};
    use tempfile::tempdir;

    #[test]
    fn collects_and_reports_stats() {
        // Setup temp dir
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path();
        let tree = FsNode::Dir(
            "root",
            vec![
                FsNode::File("file.txt", "hello"), // 5 bytes
                FsNode::Dir(
                    "subdir",
                    vec![
                        FsNode::File("file2.txt", "world!"), // 6 bytes
                        FsNode::Dir(
                            "subdir2",
                            vec![
                                FsNode::File("file3.txt", "hello world!"), // 12 bytes
                            ],
                        ),
                    ],
                ),
            ],
        );
        create_fs_tree(tmp_path, &tree).unwrap();
        let root = tmp_path.join("root");

        let stats = collect_stats(&root, None).expect("Not expecting errors for collect_stats");

        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_dirs, 2);
        assert_eq!(stats.total_size, 23);
        assert!(stats.largest_file.is_some());
        assert_eq!(
            stats.largest_file,
            Some((root.join("subdir/subdir2/file3.txt"), 12))
        );
        assert_eq!(stats.max_depth, 3);
    }

    #[test]
    fn collects_and_reports_stats_with_max_depth() {
        // Setup temp dir
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path();
        let tree = FsNode::Dir(
            "root",
            vec![
                FsNode::File("file.txt", "hello"), // 5 bytes
                FsNode::Dir(
                    "subdir",
                    vec![
                        FsNode::File("file2.txt", "world!"), // 6 bytes
                        FsNode::Dir(
                            "subdir2",
                            vec![
                                FsNode::File("file3.txt", "hello world!"), // 12 bytes
                            ],
                        ),
                    ],
                ),
            ],
        );
        create_fs_tree(tmp_path, &tree).unwrap();
        let root = tmp_path.join("root");

        let stats = collect_stats(&root, Some(2)).expect("Not expecting errors for collect_stats");

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_dirs, 2);
        assert_eq!(stats.total_size, 11);
        assert!(stats.largest_file.is_some());
        assert_eq!(stats.largest_file, Some((root.join("subdir/file2.txt"), 6)));
        assert_eq!(stats.max_depth, 2);
    }
}
