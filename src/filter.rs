use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};

pub trait PathFilter {
    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool;
}

pub struct GitIgnoreFilter {
    root: PathBuf,
    patterns: Vec<GitignorePattern>,
}

// TODO: Return Result<Self> from new()
// gitignore semantics:
// - patterns are evaluated in order
// - last match wins
// - directories match both dir and children via extra matcher
impl GitIgnoreFilter {
    pub fn new(root: &Path, patterns: &[String]) -> Self {
        let mut compiled_patterns = Vec::new();
        for pattern in patterns {
            let mut cleaned = pattern.as_str();

            // Negation
            let negated = cleaned.starts_with("!");
            if negated {
                cleaned = &cleaned[1..];
            };

            // Anchored at root
            let anchored = cleaned.starts_with("/");
            if anchored {
                cleaned = &cleaned[1..];
            }

            // Directory only
            let dir = cleaned.ends_with("/");
            if dir {
                cleaned = &cleaned[..cleaned.len() - 1];
            }

            let effective_pattern = if !anchored {
                format!("**/{}", cleaned)
            } else {
                cleaned.to_string()
            };

            // normal matcher
            compiled_patterns.push(GitignorePattern {
                matcher: match Glob::new(&effective_pattern) {
                    Ok(glob) => glob.compile_matcher(),
                    Err(e) => {
                        eprintln!("Warning: Invalid ignore pattern '{}': {}", pattern, e);
                        continue;
                    }
                },
                dir: dir,
                negated: negated,
            });

            // extra matcher for directories to include children
            if dir {
                compiled_patterns.push(GitignorePattern {
                    matcher: match Glob::new(&format!("{}/**", effective_pattern)) {
                        Ok(glob) => glob.compile_matcher(),
                        Err(e) => {
                            eprintln!("Warning: Invalid ignore pattern '{}': {}", pattern, e);
                            continue;
                        }
                    },
                    dir: false, // children can be files
                    negated: negated,
                });
            }
        }

        Self {
            root: root.to_path_buf(),
            patterns: compiled_patterns,
        }
    }
}

struct GitignorePattern {
    matcher: GlobMatcher,
    dir: bool,
    negated: bool,
}

impl PathFilter for GitIgnoreFilter {
    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let rel_path = match path.strip_prefix(&self.root) {
            Ok(p) => p,
            Err(_e) => return false,
        };
        let mut ignored = false;
        for pat in &self.patterns {
            if pat.dir && !is_dir {
                continue;
            }

            if pat.matcher.is_match(rel_path) {
                ignored = !pat.negated;
            }
        }
        ignored
    }
}
