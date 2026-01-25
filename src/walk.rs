use crate::error::FsError;
use crate::filter::PathFilter;
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

struct WalkContext<'a, V: FsVisitor> {
    max_depth: usize,
    follow_symlinks: bool,
    visited: HashSet<PathBuf>,
    filter: &'a dyn PathFilter,
    visitor: &'a mut V,
}

pub fn walk_dir<V: FsVisitor>(
    root: &Path,
    visitor: &mut V,
    filter: &dyn PathFilter,
    max_depth: Option<usize>,
    follow_symlinks: bool,
) {
    let mut context = WalkContext {
        max_depth: max_depth.unwrap_or(usize::MAX),
        follow_symlinks: follow_symlinks,
        visited: HashSet::new(),
        filter: filter,
        visitor: visitor,
    };
    walk_dir_internal(root, &mut context, 1);
}

// Walk `root` and call `f` for every entry.
fn walk_dir_internal<V: FsVisitor>(root: &Path, ctx: &mut WalkContext<V>, depth: usize) {
    let entries = match fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            ctx.visitor.on_error(FsError::Io(root.to_path_buf(), e));
            return;
        }
    };

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                ctx.visitor.on_error(FsError::Io(root.to_path_buf(), e));
                continue;
            }
        };
        let path = entry.path();

        let is_dir = match entry.file_type() {
            Ok(ft) => ft.is_dir(),
            Err(e) => {
                ctx.visitor.on_error(FsError::Io(path, e));
                continue;
            }
        };

        if ctx.filter.is_ignored(&path, is_dir) {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                ctx.visitor.on_error(FsError::Io(path.to_path_buf(), e));
                continue;
            }
        };

        if meta.is_symlink() {
            ctx.visitor.visit_symlink(&path, depth);
            if ctx.follow_symlinks {
                let target = match fs::canonicalize(&path) {
                    Ok(tg) => tg,
                    Err(e) => {
                        ctx.visitor.on_error(FsError::Io(path.to_path_buf(), e));
                        continue;
                    }
                };

                if ctx.visited.insert(target.clone()) {
                    let target_meta = match fs::metadata(&target) {
                        Ok(m) => m,
                        Err(e) => {
                            ctx.visitor.on_error(FsError::Io(target.to_path_buf(), e));
                            continue;
                        }
                    };

                    // Make sure that we don't visit ignored symlink targets
                    if ctx.filter.is_ignored(&target, target_meta.is_dir()) {
                        continue;
                    }

                    if target_meta.is_dir() {
                        ctx.visitor.enter_dir(&target, &target_meta, depth);
                        if depth < ctx.max_depth {
                            walk_dir_internal(&target, ctx, depth + 1);
                        }
                        ctx.visitor.exit_dir(&target, &target_meta, depth);
                    } else if target_meta.is_file() {
                        ctx.visitor.visit_file(&target, &target_meta, depth);
                    }
                }
            }
        } else if meta.is_dir() {
            if ctx.follow_symlinks {
                let target = match fs::canonicalize(&path) {
                    Ok(tg) => tg,
                    Err(e) => {
                        ctx.visitor.on_error(FsError::Io(path.to_path_buf(), e));
                        continue;
                    }
                };
                if !ctx.visited.insert(target) {
                    continue;
                }
            }
            ctx.visitor.enter_dir(&path, &meta, depth);
            if depth < ctx.max_depth {
                walk_dir_internal(&path, ctx, depth + 1);
            }
            ctx.visitor.exit_dir(&path, &meta, depth);
        } else if meta.is_file() {
            ctx.visitor.visit_file(&path, &meta, depth);
        }
    }
}
