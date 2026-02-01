use fsx::test_utils::{FsNode, create_fs_tree, gitignore_filter};
use fsx::{FindReport, find};
use fsx::{GitIgnoreFilter, PathFilter};
use std::path::Path;
use tempfile::tempdir;

fn run_find(root: &Path, filter: &dyn PathFilter, pattern: &str) -> FindReport {
    find(root, None, false, filter, pattern)
}

#[test]
fn finds_files_without_ignore() {
    let tmp = tempdir().unwrap();

    create_fs_tree(
        tmp.path(),
        &FsNode::Dir(
            "root",
            vec![FsNode::File("a.txt", "a"), FsNode::File("b.log", "b")],
        ),
    )
    .unwrap();

    let filter = gitignore_filter(tmp.path(), &[]);
    let report = run_find(tmp.path(), &filter, r".*");

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.entries.len(), 2);
}

#[test]
fn ignores_files_matching_pattern() {
    let tmp = tempdir().unwrap();

    create_fs_tree(
        tmp.path(),
        &FsNode::Dir(
            "root",
            vec![FsNode::File("a.txt", "a"), FsNode::File("b.log", "b")],
        ),
    )
    .unwrap();

    let filter = gitignore_filter(tmp.path(), &["*.log"]);
    let report = run_find(tmp.path(), &filter, r".*");

    assert_eq!(report.entries.len(), 1);
    assert!(report.entries[0].name.ends_with("a.txt"));
}

#[test]
fn ignores_directory_and_children() {
    let tmp = tempdir().unwrap();

    create_fs_tree(
        tmp.path(),
        &FsNode::Dir(
            "root",
            vec![
                FsNode::Dir("target", vec![FsNode::File("a.txt", "a")]),
                FsNode::File("b.txt", "b"),
            ],
        ),
    )
    .unwrap();

    let filter = gitignore_filter(tmp.path(), &["target/"]);
    let report = run_find(tmp.path(), &filter, r".*\.txt");

    assert_eq!(report.entries.len(), 1);
    assert!(report.entries[0].name.ends_with("b.txt"));
}

#[test]
fn negated_pattern_reincludes_file() {
    let tmp = tempdir().unwrap();

    create_fs_tree(
        tmp.path(),
        &FsNode::Dir(
            "root",
            vec![FsNode::File("a.log", "a"), FsNode::File("keep.log", "b")],
        ),
    )
    .unwrap();

    let filter = gitignore_filter(tmp.path(), &["*.log", "!keep.log"]);

    let report = run_find(tmp.path(), &filter, r".*\.log");

    assert_eq!(report.entries.len(), 1);
    assert!(report.entries[0].name.ends_with("keep.log"));
}

#[test]
fn cli_patterns_override_gitignore() {
    let tmp = tempdir().unwrap();

    // Create .gitignore
    std::fs::write(tmp.path().join(".gitignore"), "*.txt\n").unwrap();

    create_fs_tree(
        tmp.path(),
        &FsNode::Dir(
            "root",
            vec![FsNode::File("a.txt", "a"), FsNode::File("b.log", "b")],
        ),
    )
    .unwrap();

    let filter = GitIgnoreFilter::from_gitignore(tmp.path(), &[String::from("!a.txt")]);

    let report = run_find(tmp.path(), &filter, r".*");

    assert_eq!(report.entries.len(), 3); // .gitignore, a.txt, b.log
}

#[test]
fn invalid_regex_is_reported() {
    let tmp = tempdir().unwrap();

    let filter = gitignore_filter(tmp.path(), &[]);
    let report = find(tmp.path(), None, false, &filter, "(");

    assert_eq!(report.entries.len(), 0);
    assert_eq!(report.errors.len(), 1);
}
