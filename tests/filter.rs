use std::path::PathBuf;

use fsx::filter::{GitIgnoreFilter, PathFilter};

fn root() -> PathBuf {
    PathBuf::from("/project")
}

fn p(rel: &str) -> PathBuf {
    root().join(rel)
}

#[test]
fn no_patterns_matches_nothing() {
    let filter = GitIgnoreFilter::new(&root(), &[]);

    assert!(!filter.is_ignored(&PathBuf::from("/project/foo.txt"), false));
    assert!(!filter.is_ignored(&PathBuf::from("/project/dir/bar.txt"), false));
}

#[test]
fn ignore_single_file_by_name() {
    let patterns = vec!["foo.txt".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    assert!(filter.is_ignored(&PathBuf::from("/project/foo.txt"), false));
    assert!(!filter.is_ignored(&PathBuf::from("/project/bar.txt"), false));
}

#[test]
fn ignore_file_in_subdirectory() {
    let patterns = vec!["foo.txt".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    assert!(filter.is_ignored(&PathBuf::from("/project/subdir/foo.txt"), false));
}

#[test]
fn wildcard_matches_multiple_files() {
    let patterns = vec!["*.log".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    assert!(filter.is_ignored(&PathBuf::from("/project/a.log"), false));
    assert!(filter.is_ignored(&PathBuf::from("/project/b.log"), false));
    assert!(!filter.is_ignored(&PathBuf::from("/project/a.txt"), false));
}

#[test]
fn double_star_matches_nested_paths() {
    let patterns = vec!["**/*.rs".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    assert!(filter.is_ignored(&PathBuf::from("/project/main.rs"), false));
    assert!(filter.is_ignored(&PathBuf::from("/project/src/lib.rs"), false));
    assert!(filter.is_ignored(&PathBuf::from("/project/src/foo/bar.rs"), false));
}

#[test]
fn relative_path_is_used_for_matching() {
    let patterns = vec!["target".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    // should match relative "target", not absolute "/project/target"
    assert!(filter.is_ignored(&PathBuf::from("/project/target"), true));
    assert!(filter.is_ignored(&PathBuf::from("/project/foo/target"), true));
}

#[test]
fn outside_root_is_not_ignored() {
    let patterns = vec!["foo.txt".to_string()];
    let filter = GitIgnoreFilter::new(&root(), &patterns);

    // Defensive behavior: paths outside root should not match
    assert!(!filter.is_ignored(&PathBuf::from("/other/foo.txt"), false));
}

#[test]
fn anchored_file_matches_only_at_root() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("/foo.txt")]);

    assert!(filter.is_ignored(&p("foo.txt"), false));
    assert!(!filter.is_ignored(&p("sub/foo.txt"), false));
}

#[test]
fn unanchored_file_matches_anywhere() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("foo.txt")]);

    assert!(filter.is_ignored(&p("foo.txt"), false));
    assert!(filter.is_ignored(&p("sub/foo.txt"), false));
}

#[test]
fn anchored_directory_matches_only_at_root() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("/target")]);

    assert!(filter.is_ignored(&p("target"), true));
    assert!(!filter.is_ignored(&p("sub/target"), true));
}

#[test]
fn anchored_nested_path_matches_only_from_root() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("/foo/bar.txt")]);

    assert!(filter.is_ignored(&p("foo/bar.txt"), false));
    assert!(!filter.is_ignored(&p("x/foo/bar.txt"), false));
}

#[test]
fn anchored_pattern_does_not_match_partial_prefix() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("/foo")]);

    assert!(filter.is_ignored(&p("foo"), true));
    assert!(!filter.is_ignored(&p("foobar"), true));
    assert!(!filter.is_ignored(&p("sub/foo"), true));
}

#[test]
fn anchored_and_unanchored_can_coexist() {
    let filter = GitIgnoreFilter::new(&root(), &[String::from("/foo"), String::from("bar")]);

    assert!(filter.is_ignored(&p("foo"), true));
    assert!(!filter.is_ignored(&p("sub/foo"), true));

    assert!(filter.is_ignored(&p("bar"), true));
    assert!(filter.is_ignored(&p("sub/bar"), true));
}

#[test]
fn directory_only_pattern_matches_dirs() {
    let filter = GitIgnoreFilter::new(&root(), &["build/".into()]);

    assert!(filter.is_ignored(&p("build"), true)); // dir at root
    assert!(filter.is_ignored(&p("src/build"), true)); // dir deeper
    assert!(!filter.is_ignored(&p("build"), false)); // file ignored? NO
    assert!(filter.is_ignored(&p("src/build/main.o"), false)); // file in ignored dir
}

#[test]
fn anchored_directory_only_pattern() {
    let filter = GitIgnoreFilter::new(&root(), &["/build/".into()]);

    assert!(filter.is_ignored(&p("build"), true)); // matches root
    assert!(!filter.is_ignored(&p("src/build"), true)); // deeper does not match
}

#[test]
fn directory_only_pattern_with_glob() {
    let filter = GitIgnoreFilter::new(&root(), &["**/target/".into()]);

    assert!(filter.is_ignored(&p("target"), true));
    assert!(filter.is_ignored(&p("sub/target"), true));
    assert!(!filter.is_ignored(&p("sub/target.txt"), false));
}

#[test]
fn negation_reincludes_file() {
    let filter = GitIgnoreFilter::new(&root(), &["foo.txt".into(), "!foo.txt".into()]);

    assert!(!filter.is_ignored(&p("foo.txt"), false));
}

#[test]
fn negation_order_matters_last_wins() {
    let filter = GitIgnoreFilter::new(&root(), &["!foo.txt".into(), "foo.txt".into()]);

    assert!(filter.is_ignored(&p("foo.txt"), false));
}

#[test]
fn negation_only_affects_matching_paths() {
    let filter = GitIgnoreFilter::new(&root(), &["*.log".into(), "!important.log".into()]);

    assert!(filter.is_ignored(&p("debug.log"), false));
    assert!(!filter.is_ignored(&p("important.log"), false));
}

#[test]
fn negation_with_directories() {
    let filter = GitIgnoreFilter::new(&root(), &["build/".into(), "!build/keep/".into()]);

    assert!(filter.is_ignored(&p("build"), true));
    assert!(filter.is_ignored(&p("build/tmp"), true));
    assert!(!filter.is_ignored(&p("build/keep"), true));
    assert!(!filter.is_ignored(&p("build/keep/file.txt"), false));
}

#[test]
fn negation_does_not_match_if_parent_dir_not_ignored() {
    let filter = GitIgnoreFilter::new(&root(), &["!foo.txt".into()]);

    // Git behavior: negation without prior ignore does nothing
    assert!(!filter.is_ignored(&p("foo.txt"), false));
}

#[test]
fn anchored_negation() {
    let filter = GitIgnoreFilter::new(&root(), &["*.txt".into(), "!/keep.txt".into()]);

    assert!(!filter.is_ignored(&p("keep.txt"), false));
    assert!(filter.is_ignored(&p("sub/keep.txt"), false));
}

mod gitignore_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_gitignore(dir: &std::path::Path, contents: &str) {
        let path = dir.join(".gitignore");
        let mut file = File::create(path).unwrap();
        write!(file, "{}", contents).unwrap();
    }

    #[test]
    fn basic_gitignore_parsing() {
        let dir = tempdir().unwrap();
        write_gitignore(dir.path(), "target/\n*.log\n");

        let filter = GitIgnoreFilter::from_gitignore(dir.path(), &[]);
        let patterns: Vec<String> = filter
            .patterns()
            .iter()
            .map(|p| p.matcher().glob().to_string())
            .collect();
        assert!(patterns.iter().any(|p| p.contains("target")));
        assert!(patterns.iter().any(|p| p.contains("*.log")));
    }

    #[test]
    fn ignores_comments_and_empty_lines() {
        let dir = tempdir().unwrap();
        write_gitignore(dir.path(), "# comment\n\n*.tmp\n");

        let filter = GitIgnoreFilter::from_gitignore(dir.path(), &[]);
        let patterns: Vec<String> = filter
            .patterns()
            .iter()
            .map(|p| p.matcher().glob().to_string())
            .collect();
        assert!(patterns.iter().any(|p| p.contains("*.tmp")));
        assert!(!patterns.iter().any(|p| p.contains("#")));
    }

    #[test]
    fn cli_patterns_are_appended() {
        let dir = tempdir().unwrap();
        write_gitignore(dir.path(), "target/\n");

        let filter = GitIgnoreFilter::from_gitignore(dir.path(), &["build/".into()]);
        let patterns: Vec<String> = filter
            .patterns()
            .iter()
            .map(|p| p.matcher().glob().to_string())
            .collect();
        assert!(patterns.iter().any(|p| p.contains("target")));
        assert!(patterns.iter().any(|p| p.contains("build")));
    }

    #[test]
    fn missing_gitignore_uses_cli_patterns() {
        let dir = tempdir().unwrap();
        let filter = GitIgnoreFilter::from_gitignore(dir.path(), &["build/".into()]);
        let patterns: Vec<String> = filter
            .patterns()
            .iter()
            .map(|p| p.matcher().glob().to_string())
            .collect();
        assert!(patterns.iter().any(|p| p.contains("build")));
    }

    #[test]
    fn wildcard_ignore_with_unanchored_negation_reincludes_at_any_depth() {
        let filter = GitIgnoreFilter::new(&root(), &["*.txt".into(), "!a.txt".into()]);

        assert!(!filter.is_ignored(&p("a.txt"), false));
        assert!(!filter.is_ignored(&p("sub/a.txt"), false));

        assert!(filter.is_ignored(&p("b.txt"), false));
        assert!(filter.is_ignored(&p("sub/b.txt"), false));
    }
}
