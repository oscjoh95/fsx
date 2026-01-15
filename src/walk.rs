use crate::error::FsError;
use std::{
    collections::HashSet,
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

pub trait FsVisitor {
    fn visit_file(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn enter_dir(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn exit_dir(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn visit_symlink(&mut self, path: &Path, depth: usize);
    fn on_error(&mut self, error: FsError);
}

pub fn walk_dir<T: FsVisitor>(
    root: &Path,
    visitor: &mut T,
    max_depth: Option<usize>,
    follow_symlinks: bool,
) {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    walk_dir_internal(
        root,
        visitor,
        1,
        max_depth.unwrap_or(usize::MAX),
        follow_symlinks,
        &mut visited,
    );
}

// Walk `root` and call `f` for every entry.
fn walk_dir_internal<T: FsVisitor>(
    root: &Path,
    visitor: &mut T,
    depth: usize,
    max_depth: usize,
    follow_symlinks: bool,
    visited: &mut HashSet<PathBuf>, // tracks canonicalized symlink targets
) {
    let entries = match fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            visitor.on_error(FsError::Io(root.to_path_buf(), e));
            return;
        }
    };

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                visitor.on_error(FsError::Io(root.to_path_buf(), e));
                continue;
            }
        };
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                visitor.on_error(FsError::Io(path.to_path_buf(), e));
                continue;
            }
        };

        if meta.is_symlink() {
            visitor.visit_symlink(&path, depth);
            if follow_symlinks {
                let target = match fs::canonicalize(&path) {
                    Ok(tg) => tg,
                    Err(e) => {
                        visitor.on_error(FsError::Io(path.to_path_buf(), e));
                        continue;
                    }
                };
                if visited.insert(target.clone()) {
                    let target_meta = match fs::metadata(&target) {
                        Ok(m) => m,
                        Err(e) => {
                            visitor.on_error(FsError::Io(target.to_path_buf(), e));
                            continue;
                        }
                    };
                    visitor.enter_dir(&target, &target_meta, depth);
                    walk_dir_internal(
                        &target,
                        visitor,
                        depth + 1,
                        max_depth,
                        follow_symlinks,
                        visited,
                    );
                }
            }
        } else if meta.is_dir() {
            if follow_symlinks {
                let target = match fs::canonicalize(&path) {
                    Ok(tg) => tg,
                    Err(e) => {
                        visitor.on_error(FsError::Io(path.to_path_buf(), e));
                        continue;
                    }
                };
                if !visited.insert(target) {
                    continue;
                }
            }
            visitor.enter_dir(&path, &meta, depth);
            if depth < max_depth {
                walk_dir_internal(
                    &path,
                    visitor,
                    depth + 1,
                    max_depth,
                    follow_symlinks,
                    visited,
                );
            }
            visitor.exit_dir(&path, &meta, depth);
        } else if meta.is_file() {
            visitor.visit_file(&path, &meta, depth);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FsNode, create_fs_tree};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[derive(Default)]
    struct WalkTestVisitor {
        seen_files: Vec<(PathBuf, usize)>,
        seen_dirs_enter: Vec<(PathBuf, usize)>,
        seen_dirs_exit: Vec<(PathBuf, usize)>,
    }

    impl FsVisitor for WalkTestVisitor {
        fn visit_file(&mut self, path: &Path, _meta: &Metadata, depth: usize) {
            self.seen_files.push((path.to_path_buf(), depth));
        }

        fn enter_dir(&mut self, path: &Path, _meta: &Metadata, depth: usize) {
            self.seen_dirs_enter.push((path.to_path_buf(), depth));
        }

        fn exit_dir(&mut self, path: &Path, _meta: &Metadata, depth: usize) {
            self.seen_dirs_exit.push((path.to_path_buf(), depth));
        }

        fn visit_symlink(&mut self, _path: &Path, _depth: usize) {}

        fn on_error(&mut self, error: FsError) {
            panic!("Unexpected error: {:?}", error);
        }
    }

    #[test]
    fn walks_directory_and_reports_depth() {
        // Setup temp dir
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path();
        let tree = FsNode::Dir(
            "root",
            vec![
                FsNode::File("file.txt", "hello"),
                FsNode::Dir("subdir", vec![FsNode::File("file2.txt", "world")]),
            ],
        );
        create_fs_tree(tmp_path, &tree).unwrap();
        let root = tmp_path.join("root");

        let mut visitor = WalkTestVisitor::default();

        // Walk
        walk_dir(&root, &mut visitor, None, false);

        // Total entries
        assert_eq!(visitor.seen_files.len(), 2);
        assert_eq!(visitor.seen_dirs_enter.len(), 1);

        // We should enter and exit the same directories. Sort so we can compare them easily
        visitor.seen_dirs_enter.sort();
        visitor.seen_dirs_exit.sort();
        assert_eq!(visitor.seen_dirs_enter, visitor.seen_dirs_exit);

        // file.txt
        assert!(visitor.seen_files.contains(&(root.join("file.txt"), 1)));

        // subdir
        assert!(visitor.seen_dirs_enter.contains(&(root.join("subdir"), 1)));

        // subdir/file2.txt
        assert!(
            visitor
                .seen_files
                .contains(&(root.join("subdir/file2.txt"), 2))
        );
    }

    #[test]
    fn walks_directories_to_max_depth() {
        // Setup temp dir
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path();
        let tree = FsNode::Dir(
            "root",
            vec![
                FsNode::File("file.txt", "hello"),
                FsNode::Dir(
                    "subdir",
                    vec![
                        FsNode::File("file2.txt", "world"),
                        FsNode::Dir("subdir2", vec![FsNode::File("file3.txt", "!")]),
                    ],
                ),
            ],
        );
        create_fs_tree(tmp_path, &tree).unwrap();
        let root = tmp_path.join("root");

        let mut visitor = WalkTestVisitor::default();

        // Walk without max depth
        walk_dir(&root, &mut visitor, Some(2), false);

        // We should enter and exit the same directories. Sort so we can compare them easily
        visitor.seen_dirs_enter.sort();
        visitor.seen_dirs_exit.sort();
        assert_eq!(visitor.seen_dirs_enter, visitor.seen_dirs_exit);

        // max depth 2 should see root and subdir but not what's inside subdir2
        assert!(visitor.seen_files.contains(&(root.join("file.txt"), 1)));
        assert!(visitor.seen_dirs_enter.contains(&(root.join("subdir"), 1)));
        assert!(
            visitor
                .seen_files
                .contains(&(root.join("subdir/file2.txt"), 2))
        );
        assert!(
            visitor
                .seen_dirs_enter
                .contains(&(root.join("subdir/subdir2"), 2))
        );
        // Assert that no depth larger than 2 and that file3.txt is not found
        for (path, depth) in visitor.seen_files {
            assert!(depth <= 2);
            assert!(path != root.join("subdir/subdir2/file3.txt"));
        }
        for (_path, depth) in visitor.seen_dirs_enter {
            assert!(depth <= 2);
        }
    }

    #[cfg(target_os = "windows")]
    mod windows_only_tests {
        use crate::error::FsError;
        use crate::test_utils::{FsNode, create_fs_tree};
        use crate::walk::{FsVisitor, walk_dir};
        use std::fs;
        use std::path::{Path, PathBuf};
        use tempfile::tempdir;

        #[derive(Default)]
        struct SymlinkTestVisitor {
            files: Vec<(PathBuf, usize)>,
            dirs: Vec<(PathBuf, usize)>,
            symlinks: Vec<(PathBuf, usize)>,
        }

        impl FsVisitor for SymlinkTestVisitor {
            fn visit_file(&mut self, path: &Path, _meta: &fs::Metadata, depth: usize) {
                self.files.push((path.to_path_buf(), depth));
            }

            fn enter_dir(&mut self, path: &Path, _meta: &fs::Metadata, depth: usize) {
                self.dirs.push((path.to_path_buf(), depth));
            }

            fn exit_dir(&mut self, _path: &Path, _meta: &fs::Metadata, _depth: usize) {}

            fn visit_symlink(&mut self, path: &Path, depth: usize) {
                self.symlinks.push((path.to_path_buf(), depth));
            }

            fn on_error(&mut self, err: FsError) {
                panic!("Unexpected error: {:?}", err);
            }
        }

        #[test]
        fn detects_symlinks_but_does_not_follow_by_default() {
            let tmp = tempdir().unwrap();
            let tmp_path = tmp.path();

            let tree = FsNode::Dir(
                "root",
                vec![FsNode::Dir(
                    "subdir",
                    vec![
                        FsNode::File("file.txt", "hello"),
                        FsNode::SymlinkDir("link_to_subdir", "subdir"),
                    ],
                )],
            );

            create_fs_tree(tmp_path, &tree).unwrap();
            let root = tmp_path.join("root");

            let mut visitor = SymlinkTestVisitor::default();
            walk_dir(&root, &mut visitor, None, false);

            // subdir entered once
            assert_eq!(
                visitor
                    .dirs
                    .iter()
                    .filter(|(p, _)| p.ends_with("subdir"))
                    .count(),
                1
            );

            // symlink detected
            assert!(
                visitor
                    .symlinks
                    .iter()
                    .any(|(p, _)| p.ends_with("link_to_subdir"))
            );

            // file only seen once
            assert_eq!(
                visitor
                    .files
                    .iter()
                    .filter(|(p, _)| p.ends_with("file.txt"))
                    .count(),
                1
            );
        }

        #[test]
        fn symlink_cycle_does_not_revisit_directories() {
            let tmp = tempdir().unwrap();
            let tmp_path = tmp.path();

            let tree = FsNode::Dir(
                "a",
                vec![FsNode::Dir(
                    "b",
                    vec![FsNode::Dir(
                        "c",
                        vec![FsNode::SymlinkDir("d", "../../../a")],
                    )],
                )],
            );

            create_fs_tree(tmp_path, &tree).unwrap();
            let start = tmp_path.join("a/b");

            let mut visitor = SymlinkTestVisitor::default();
            walk_dir(&start, &mut visitor, None, true);

            // Each directory should be entered exactly once
            let mut dirs = visitor.dirs.clone();
            dirs.sort();
            dirs.dedup();

            assert_eq!(
                dirs.len(),
                visitor.dirs.len(),
                "Directory visited more than once"
            );

            assert!(
                visitor.dirs.iter().any(|(p, _)| p.ends_with("a")),
                "Expected to reach directory 'a' via symlink"
            );
        }
    }
}
