use crate::error::FsError;
use std::{
    fs::{self, Metadata},
    path::Path,
};

pub trait FsVisitor {
    fn visit_file(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn enter_dir(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn exit_dir(&mut self, path: &Path, meta: &Metadata, depth: usize);
    fn on_error(&mut self, error: FsError);
}

pub fn walk_dir<T: FsVisitor>(root: &Path, visitor: &mut T, max_depth: Option<usize>) {
    walk_dir_internal(root, visitor, 1, max_depth.unwrap_or(usize::MAX));
}

// Walk `root` and call `f` for every entry.
fn walk_dir_internal<T: FsVisitor>(root: &Path, visitor: &mut T, depth: usize, max_depth: usize) {
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

        if meta.is_dir() {
            visitor.enter_dir(&path, &meta, depth);
            if depth < max_depth {
                walk_dir_internal(&path, visitor, depth + 1, max_depth);
            }
            visitor.exit_dir(&path, &meta, depth);
        } else if meta.is_file() {
            visitor.visit_file(&path, &meta, depth);
        }
        // TODO: Handle symlinks in else here
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

        fn on_error(&mut self, _error: FsError) {
            panic!("did not expect an error");
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
        walk_dir(&root, &mut visitor, None);

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
        walk_dir(&root, &mut visitor, Some(2));

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
}
