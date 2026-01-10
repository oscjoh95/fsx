use crate::error::FsError;
use std::{fs, path::Path};

pub enum EntryType {
    File,
    Dir,
    Symlink,
}

impl EntryType {
    fn from_metadata(meta: &std::fs::Metadata) -> Self {
        if meta.is_dir() {
            return Self::Dir;
        } else if meta.is_file() {
            return Self::File;
        } else {
            return Self::Symlink;
        }
    }
}

pub fn walk_dir<F, G>(root: &Path, on_entry: &mut F, on_error: &mut G, max_depth: Option<usize>)
where
    F: FnMut(&Path, &std::fs::Metadata, EntryType, usize),
    G: FnMut(FsError),
{
    walk_dir_internal(root, on_entry, on_error, 1, max_depth.unwrap_or(usize::MAX));
}

// Walk `root` and call `f` for every entry.
fn walk_dir_internal<F, G>(
    root: &Path,
    on_entry: &mut F,
    on_error: &mut G,
    depth: usize,
    max_depth: usize,
) where
    F: FnMut(&Path, &std::fs::Metadata, EntryType, usize),
    G: FnMut(FsError),
{
    let entries = match fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            on_error(FsError::Io(root.to_path_buf(), e));
            return;
        }
    };

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                on_error(FsError::Io(root.to_path_buf(), e));
                continue;
            }
        };
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                on_error(FsError::Io(path.to_path_buf(), e));
                continue;
            }
        };

        // Call function for entry
        on_entry(&path, &meta, EntryType::from_metadata(&meta), depth);

        // continue recursing for directories
        if meta.is_dir() && depth < max_depth {
            walk_dir_internal(&path, on_entry, on_error, depth + 1, max_depth);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{FsNode, create_fs_tree};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::tempdir;

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

        // Collect results
        let mut seen: Vec<(PathBuf, EntryType, usize)> = Vec::new();

        let mut on_entry =
            |p: &Path, _meta: &std::fs::Metadata, entry_type: EntryType, depth: usize| {
                seen.push((p.to_path_buf(), entry_type, depth));
            };

        let mut on_error = |_err: FsError| {
            panic!("did not expect an error");
        };

        // Walk
        walk_dir(&root, &mut on_entry, &mut on_error, None);

        // Build a lookup tree
        let mut map: HashMap<PathBuf, (EntryType, usize)> = HashMap::new();
        for (path, ty, depth) in seen {
            map.insert(path, (ty, depth));
        }

        // Helper paths
        let file1 = root.join("file.txt");
        let subdir = root.join("subdir");
        let file2 = root.join("subdir/file2.txt");

        // Total entries
        assert_eq!(map.len(), 3);

        // file.txt
        let (ty, depth) = map.get(&file1).expect("Missing file.txt");
        assert!(matches!(ty, EntryType::File));
        assert_eq!(*depth, 1);

        // subdir
        let (ty, depth) = map.get(&subdir).expect("Missing subdir");
        assert!(matches!(ty, EntryType::Dir));
        assert_eq!(*depth, 1);

        // subdir/file2.txt
        let (ty, depth) = map.get(&file2).expect("Missing subdir/file2.txt");
        assert!(matches!(ty, EntryType::File));
        assert_eq!(*depth, 2);
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

        // Collect results
        let mut seen: Vec<(PathBuf, EntryType, usize)> = Vec::new();

        let mut on_entry =
            |p: &Path, _meta: &std::fs::Metadata, entry_type: EntryType, depth: usize| {
                seen.push((p.to_path_buf(), entry_type, depth));
            };

        let mut on_error = |_err: FsError| {
            panic!("did not expect an error");
        };

        // Walk without max depth
        walk_dir(&root, &mut on_entry, &mut on_error, Some(2));

        let mut map: HashMap<PathBuf, (EntryType, usize)> = HashMap::new();
        for (path, ty, depth) in seen {
            map.insert(path, (ty, depth));
        }

        // max depth 2 should see root and subdir but not what's inside subdir2
        assert!(map.contains_key(&root.join("file.txt")));
        assert!(map.contains_key(&root.join("subdir")));
        assert!(map.contains_key(&root.join("subdir/file2.txt")));
        assert!(map.contains_key(&root.join("subdir/subdir2")));
        for (_ty, depth) in map.values() {
            assert!(*depth <= 2);
        }
        let file3 = root.join("subdir/subdir2/file3.txt");
        assert!(
            !map.contains_key(&file3),
            "file3.txt should not be visited due to max_depth"
        );
    }
}
