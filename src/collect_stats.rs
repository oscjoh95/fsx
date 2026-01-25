use crate::error::FsError;
use crate::filter::PathFilter;
use crate::walk::{FsVisitor, walk_dir};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct FsStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_symlinks: usize,
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

    fn visit_symlink(&mut self, _path: &Path, _depth: usize) {
        self.stats.total_symlinks += 1;
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

pub fn collect(
    root: &Path,
    max_depth: Option<usize>,
    follow_symlinks: bool,
    filter: &dyn PathFilter,
) -> FsStatsReport {
    let mut visitor = StatsVisitor::default();

    walk_dir(root, &mut visitor, filter, max_depth, follow_symlinks);

    visitor.into_report()
}
