pub mod error; // FsError etc
pub mod filter;
pub mod matcher;
pub mod test_utils; // helpers for tests
pub mod walk; // filesystem walking

// Each command gets its own module, re-exported for easier access
pub mod collect_stats;
pub mod find;

// Re-export the main API at the crate root
pub use collect_stats::collect;
pub use filter::{PathFilter, GitIgnoreFilter};
pub use find::{FindReport, find};
pub use walk::{FsVisitor, walk_dir};
