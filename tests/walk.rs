use fsx::error::FsError;
use fsx::{
    FsVisitor, PathFilter,
    test_utils::{FsNode, create_fs_tree},
    walk_dir,
};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
struct TestPathFilter {
    ignored_paths: Vec<PathBuf>,
}

impl PathFilter for TestPathFilter {
    fn is_ignored(&self, path: &Path, _is_dir: bool) -> bool {
        self.ignored_paths.iter().any(|p| p.as_path() == path)
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
    let filter = TestPathFilter::default();

    // Walk
    walk_dir(&root, &mut visitor, &filter, None, false);

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
    let filter = TestPathFilter::default();

    // Walk without max depth
    walk_dir(&root, &mut visitor, &filter, Some(2), false);

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

/*
Ignore related tests
 */
#[test]
fn ignore_single_file() {
    // Setup temp dir
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path();
    let tree = FsNode::Dir("root", vec![FsNode::File("file.txt", "hello")]);
    create_fs_tree(tmp_path, &tree).unwrap();
    let root = tmp_path.join("root");

    let mut visitor = WalkTestVisitor::default();
    let filter = TestPathFilter {
        ignored_paths: vec![root.join("file.txt")],
    };

    walk_dir(&root, &mut visitor, &filter, None, false);

    assert_eq!(visitor.seen_dirs_enter.len(), 0);
    assert!(!visitor.seen_files.contains(&(root.join("file.txt"), 1)));
}

#[test]
fn ignore_single_directory() {
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
                    FsNode::Dir("subdir2", Vec::new()),
                ],
            ),
        ],
    );
    create_fs_tree(tmp_path, &tree).unwrap();
    let root = tmp_path.join("root");

    let mut visitor = WalkTestVisitor::default();
    let filter = TestPathFilter {
        ignored_paths: vec![root.join("subdir").join("subdir2")],
    };

    walk_dir(&root, &mut visitor, &filter, None, false);

    assert_eq!(visitor.seen_dirs_enter.len(), 1);
    assert!(visitor.seen_dirs_enter.contains(&(root.join("subdir"), 1)));
    assert_eq!(visitor.seen_files.len(), 2);
    assert!(visitor.seen_files.contains(&(root.join("file.txt"), 1)));
    assert!(
        visitor
            .seen_files
            .contains(&(root.join("subdir").join("file2.txt"), 2))
    );
}

#[test]
fn ignore_nested_directory() {
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
    let filter = TestPathFilter {
        ignored_paths: vec![root.join("subdir")],
    };

    walk_dir(&root, &mut visitor, &filter, None, false);

    assert_eq!(visitor.seen_dirs_enter.len(), 0);
    assert_eq!(visitor.seen_files.len(), 1);
    assert!(visitor.seen_files.contains(&(root.join("file.txt"), 1)));
}

#[test]
fn ignore_nothing() {
    #[derive(Default)]
    struct FalsePathFilter {}
    impl PathFilter for FalsePathFilter {
        fn is_ignored(&self, _path: &Path, _is_dir: bool) -> bool {
            false
        }
    }

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
    let filter = FalsePathFilter::default();

    walk_dir(&root, &mut visitor, &filter, None, true);

    assert_eq!(visitor.seen_dirs_enter.len(), 2);
    assert!(visitor.seen_dirs_enter.contains(&(root.join("subdir"), 1)));
    assert!(
        visitor
            .seen_dirs_enter
            .contains(&(root.join("subdir").join("subdir2"), 2))
    );
    assert_eq!(visitor.seen_files.len(), 3);
    assert!(visitor.seen_files.contains(&(root.join("file.txt"), 1)));
    assert!(
        visitor
            .seen_files
            .contains(&(root.join("subdir").join("file2.txt"), 2))
    );
    assert!(
        visitor
            .seen_files
            .contains(&(root.join("subdir").join("subdir2").join("file3.txt"), 3))
    );
}

#[test]
fn ignore_everything() {
    #[derive(Default)]
    struct TruePathFilter {}
    impl PathFilter for TruePathFilter {
        fn is_ignored(&self, _path: &Path, _is_dir: bool) -> bool {
            true
        }
    }

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
    let filter = TruePathFilter::default();

    walk_dir(&root, &mut visitor, &filter, None, true);

    assert_eq!(visitor.seen_dirs_enter.len(), 0);
    assert_eq!(visitor.seen_files.len(), 0);
}

/*
Symlink related tests
*/
#[derive(Default, Debug)]
struct SymlinkTestVisitor {
    files: Vec<(PathBuf, usize)>,
    dirs: Vec<(PathBuf, usize)>,
    symlinks: Vec<(PathBuf, usize)>,
}

impl FsVisitor for SymlinkTestVisitor {
    fn visit_file(&mut self, path: &Path, _meta: &Metadata, depth: usize) {
        self.files.push((path.to_path_buf(), depth));
    }

    fn enter_dir(&mut self, path: &Path, _meta: &Metadata, depth: usize) {
        self.dirs.push((path.to_path_buf(), depth));
    }

    fn exit_dir(&mut self, _path: &Path, _meta: &Metadata, _depth: usize) {}

    fn visit_symlink(&mut self, path: &Path, depth: usize) {
        self.symlinks.push((path.to_path_buf(), depth));
    }

    fn on_error(&mut self, err: FsError) {
        panic!("Unexpected error: {:?}", err);
    }
}

#[cfg(target_os = "windows")]
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
    let filter = TestPathFilter::default();

    walk_dir(&root, &mut visitor, &filter, None, false);

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

#[cfg(target_os = "windows")]
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
    let filter = TestPathFilter::default();

    walk_dir(&start, &mut visitor, &filter, None, true);

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

#[cfg(target_os = "windows")]
#[test]
fn ignore_symlink_file() {
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
                    FsNode::Dir(
                        "subdir2",
                        vec![FsNode::SymlinkFile("file3.txt", "../../file.txt")],
                    ),
                ],
            ),
        ],
    );
    create_fs_tree(tmp_path, &tree).unwrap();
    let root = tmp_path.join("root");
    let start = root.join("subdir");

    let mut visitor = SymlinkTestVisitor::default();
    let filter = TestPathFilter {
        ignored_paths: vec![root.join("file.txt").canonicalize().unwrap()],
    };

    walk_dir(&start, &mut visitor, &filter, None, true);

    // We should not see file1.txt even though we have a symlink to it
    assert_eq!(visitor.dirs.len(), 1);
    assert!(
        visitor
            .dirs
            .contains(&(root.join("subdir").join("subdir2"), 1))
    );
    assert_eq!(visitor.files.len(), 1);
    assert!(
        visitor
            .files
            .contains(&(root.join("subdir").join("file2.txt"), 1))
    );
    assert_eq!(visitor.symlinks.len(), 1);
    assert!(
        visitor
            .symlinks
            .contains(&(root.join("subdir").join("subdir2").join("file3.txt"), 2))
    );
}

#[cfg(target_os = "windows")]
#[test]
fn ignore_symlink_dir() {
    // Setup temp dir
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path();
    let tree = FsNode::Dir(
        "root",
        vec![
            FsNode::Dir(
                "symlinked_dir",
                vec![
                    FsNode::File("File4.txt", "Some content"),
                    FsNode::File("File5.txt", "Some more content"),
                ],
            ),
            FsNode::Dir(
                "subdir",
                vec![
                    FsNode::File("file2.txt", "world"),
                    FsNode::Dir(
                        "subdir2",
                        vec![FsNode::SymlinkDir("dir_link", "../../symlinked_dir")],
                    ),
                ],
            ),
        ],
    );
    create_fs_tree(tmp_path, &tree).unwrap();
    let root = tmp_path.join("root");
    let start = root.join("subdir");

    let mut visitor = SymlinkTestVisitor::default();
    let filter = TestPathFilter {
        ignored_paths: vec![root.join("symlinked_dir").canonicalize().unwrap()],
    };

    walk_dir(&start, &mut visitor, &filter, None, true);

    // We should not see anything inside the symlinked dir
    assert_eq!(visitor.dirs.len(), 1);
    assert!(
        visitor
            .dirs
            .contains(&(root.join("subdir").join("subdir2"), 1))
    );
    assert_eq!(visitor.files.len(), 1);
    assert!(
        visitor
            .files
            .contains(&(root.join("subdir").join("file2.txt"), 1))
    );
    assert_eq!(visitor.symlinks.len(), 1);
    assert!(
        visitor
            .symlinks
            .contains(&(root.join("subdir").join("subdir2").join("dir_link"), 2))
    );
}
