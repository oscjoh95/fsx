use crate::error::FsError;
use crate::filter::PathFilter;
use crate::walk::{FsVisitor, walk_dir};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct FindEntry {
    pub name: PathBuf,
    pub size: u64,
    pub depth: usize,
}

#[derive(Debug)]
pub struct FindReport {
    pub entries: Vec<FindEntry>,
    pub errors: Vec<FsError>,
}

struct FindVisitor {
    pattern: Regex,
    entries: Vec<FindEntry>,
    errors: Vec<FsError>,
}

impl FindVisitor {
    pub fn new(regex: &str) -> Result<Self, FsError> {
        let re = Regex::new(regex)?;
        Ok(Self {
            pattern: re,
            entries: Vec::new(),
            errors: Vec::new(),
        })
    }
}

impl FsVisitor for FindVisitor {
    fn visit_file(&mut self, path: &Path, meta: &fs::Metadata, depth: usize) {
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            return;
        };
        if self.pattern.is_match(file_name) {
            self.entries.push(
                FindEntry { name: path.to_path_buf(), size: meta.len(), depth: depth }
            );
        }
    }

    fn enter_dir(&mut self, _path: &Path, _meta: &fs::Metadata, _depth: usize) {
        /* TODO: Add support for searching for directories */
    }

    fn exit_dir(&mut self, _path: &Path, _meta: &fs::Metadata, _depth: usize) {}

    fn visit_symlink(&mut self, _path: &Path, _depth: usize) {}

    fn on_error(&mut self, error: FsError) {
        self.errors.push(error);
    }
}

impl FindVisitor {
    fn into_report(self) -> FindReport {
        FindReport {
            entries: self.entries,
            errors: self.errors,
        }
    }
}

pub fn find(
    root: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    ignore_filter: &dyn PathFilter,
    pattern: &str,
) -> FindReport {
    let mut visitor = match FindVisitor::new(pattern) {
        Ok(vis) => vis,
        Err(err) => {
            return FindReport {
                entries: Vec::new(),
                errors: vec![err],
            };
        }
    };

    walk_dir(
        root,
        &mut visitor,
        ignore_filter,
        max_depth,
        follow_symlinks,
    );

    visitor.into_report()
}
