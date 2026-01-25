use fsx::{collect, filter::GitIgnoreFilter, test_utils::{FsNode, create_fs_tree}};
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
    let filter = GitIgnoreFilter::new(&root, &Vec::new());

    let stats = collect(&root, None, false, &filter).stats;

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
    let filter = GitIgnoreFilter::new(&root, &Vec::new());

    let stats = collect(&root, Some(2), false, &filter).stats;

    assert_eq!(stats.total_files, 2);
    assert_eq!(stats.total_dirs, 2);
    assert_eq!(stats.total_size, 11);
    assert!(stats.largest_file.is_some());
    assert_eq!(stats.largest_file, Some((root.join("subdir/file2.txt"), 6)));
    assert_eq!(stats.max_depth, 2);
}

#[cfg(target_os = "windows")]
#[test]
fn collects_and_reports_stats_with_symlinks() {
    // Setup temp dir
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path();

    let tree = FsNode::Dir(
        "root",
        vec![
            FsNode::File("file1.txt", "hello"),
            FsNode::Dir(
                "subdir",
                vec![
                    FsNode::File("file2.txt", "world!"),
                    FsNode::SymlinkFile("link_to_file2.txt", "file2.txt"),
                    FsNode::SymlinkDir("link_to_subdir", "subdir"),
                ],
            ),
        ],
    );

    create_fs_tree(tmp_path, &tree).unwrap();
    let root = tmp_path.join("root");
    let filter = GitIgnoreFilter::new(&root, &Vec::new());

    let stats = collect(&root, None, false, &filter).stats;

    // Normal files
    assert_eq!(stats.total_files, 2);
    assert_eq!(stats.total_dirs, 1);
    assert_eq!(stats.total_size, 11);
    assert_eq!(stats.total_symlinks, 2); // File and dir symlinks are counted as equivalent
    assert!(stats.largest_file.is_some());
    assert_eq!(stats.largest_file, Some((root.join("subdir/file2.txt"), 6)));
    assert_eq!(stats.max_depth, 2);
}
