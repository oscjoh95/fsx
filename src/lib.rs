pub mod walk;      // filesystem walking
pub mod error;     // FsError etc
pub mod test_utils; // helpers for tests

// Each command gets its own module, re-exported for easier access
pub mod collect_stats; // command-specific module

// Re-export the main API at the crate root
pub use walk::{walk_dir, FsVisitor, PathFilter};
pub use collect_stats::collect; // so we can call fsx::collect() directly
