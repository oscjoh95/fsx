use crate::error::FsError;
use crate::walk::{self, FsVisitor};
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

pub struct FsStatsReport {
    pub stats: FsStats,
    pub errors: Vec<FsError>,
}

#[derive(Default)]
struct StatsVisitor {
    stats: FsStats,
    errs: Vec<FsError>,
}

impl FsVisitor for StatsVisitor {
    fn visit_file(&mut self, path: &Path, meta: &fs::Metadata, depth: usize) {
        let size = meta.len();
        self.stats.total_files += 1;
        self.stats.total_size += size;
        if self
            .stats
            .largest_file
            .as_ref()
            .map_or(true, |(_, largest_size)| size > *largest_size)
        {
            self.stats.largest_file = Some((path.to_path_buf(), size));
        }
        self.stats.max_depth = self.stats.max_depth.max(depth);
    }

    fn enter_dir(&mut self, _path: &Path, _meta: &fs::Metadata, depth: usize) {
        self.stats.total_dirs += 1;
        self.stats.max_depth = self.stats.max_depth.max(depth);
    }

    fn exit_dir(&mut self, _path: &Path, _meta: &fs::Metadata, _depth: usize) {
        /* We do all work when entering dir */
    }

    fn on_error(&mut self, error: FsError) {
        self.errs.push(error);
    }
}

impl StatsVisitor {
    fn into_report(self) -> FsStatsReport {
        FsStatsReport {
            stats: self.stats,
            errors: self.errs,
        }
    }
}

pub fn collect_stats(
    root: &Path,
    max_depth: Option<usize>,
) -> FsStatsReport {
    let mut visitor = StatsVisitor::default();

    walk::walk_dir(root, &mut visitor, max_depth);
    visitor.into_report()
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

        let stats = collect_stats(&root, None).stats;

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

        let stats = collect_stats(&root, Some(2)).stats;

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_dirs, 2);
        assert_eq!(stats.total_size, 11);
        assert!(stats.largest_file.is_some());
        assert_eq!(stats.largest_file, Some((root.join("subdir/file2.txt"), 6)));
        assert_eq!(stats.max_depth, 2);
    }
}
