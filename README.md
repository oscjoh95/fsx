# fsx - File System Explorer

`fsx` is a small Rust CLI tool for exploring directory trees and reporting filesystem statistics.

It recursively walks a directory tree and computes:

- Total files
- Total directories
- Total size
- Largest file
- Maximum depth reached during traversal

It continues on errors, reporting unreadable files or directories as warnings.

---

## Installation

Build from source with Rust:

```bash
git clone https://github.com/oscjoh95/fsx.git
cd fsx
cargo build --release
``` 
The binary will be available at target/release/fsx.

## Usage
### Basic
```bash
fsx stats
```
Analyzes the current directory.

### Specify a path
```bash
fsx stats /path/to/dir
```

### Limit recursion depth
```bash
fsx stats --max-depth 2
```
Depth starts at 1 for entries directly under the root path.

If not set, the entire directory tree is traversed.

### Follow symbolic links

By default, symbolic links are detected but not followed.

To recurse into symlink targets:
```bash
fsx stats --follow-symlinks
```
Symlink cycles are detected and avoided automatically.

### Ignore paths

`fsx` automatically reads a `.gitignore` file in the root of the directory being analyzed. Patterns from this file are applied during traversal.  

You can also provide additional ignore patterns via the CLI `--ignore` flag. CLI patterns are appended to `.gitignore` patterns, so they take precedence if there’s overlap.  

```bash
fsx stats --ignore "target/"
```

Ignored directories are skipped entirely, including all children.  

Currently, only a `.gitignore` in the root directory is supported; nested `.gitignore` files are ignored.

### Ignore semantics and directory traversal

Ignore patterns are evaluated in order, following gitignore-style rules.
However, fsx applies ignores **during directory traversal**.

If a directory is ignored, fsx will not descend into it.
This means that negation patterns (`!`) cannot re-include files or
subdirectories inside an ignored directory, because they are never visited.

For example:
```bash
fsx stats --ignore "target/" --ignore "!target/keep/"
```

In this case, `target/` is ignored and traversal stops at that directory.
`target/keep/` will not be visited, even though it is later negated.

To include a subdirectory, the parent directory must not be ignored.

To selectively ignore contents of a directory while keeping a subdirectory,
ignore the contents instead of the directory itself, for example:
```bash
fsx stats --ignore "target/*" --ignore "!target/keep/"
```

### Output formats
```bash 
 fsx stats --format human # Default, human-readable sizes
 fsx stats --format raw # Exact byte counts
 fsx stats --format debug # Rust struct dump
 ```

## Options & Notes
### Arguments
```bash
[PATH] Root directory to analyze (default: .)
```
### Options
```bash
-m, --max-depth <MAX_DEPTH> Limit recursion to a maximum depth.
                            Depth starts at 1 for entries directly under PATH.
                            If not set, the entire directory tree is traversed.

--format <FORMAT> Output format:
    human - Human-readable sizes (default)
    raw - Exact byte counts
    debug - Debug output (Rust struct dump)

--follow-symlinks
    Recurse into symbolic links.
    Symlink cycles are detected and avoided.

--ignore <PATTERN>
    Ignore paths matching the given gitignore pattern.
```

### Examples
```bash 
# Analyze current directory with default human-readable output
fsx stats

# Analyze specific directory with raw output
fsx stats /path/to/dir --format raw

# Limit depth to 2
fsx stats /path/to/dir --max-depth 2

# Follow symlinks but ignore build artifacts
fsx stats --follow-symlinks --ignore "target/" --ignore "*.log"
```