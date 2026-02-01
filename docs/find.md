# fsx find

The `fsx find` command searches for files in a directory tree matching a given regex pattern.  

It uses the same traversal engine as `fsx stats` and respects `.gitignore` patterns in the root of the analyzed directory.

---

## Basic Usage

Search the current directory for all files:
```bash
fsx find --regex ".*"  
```
Search a specific directory for Rust files:
```bash
fsx find /path/to/dir --regex ".*\.rs$"  
```
---

## Options

- `--regex <PATTERN>`: Regex pattern to match file names. Defaults to `.*` (all files).  
- `-m, --max-depth <MAX_DEPTH>`: Limit recursion to a maximum depth. Depth starts at 1 for entries directly under PATH. If not set, the entire tree is traversed.  
- `--follow-symlinks`: Recurse into symbolic links. Cycles are detected automatically.  
- `-i, --ignore <PATTERN>`: Ignore files or directories matching the given pattern. CLI ignore patterns are appended to `.gitignore` patterns, taking precedence.  
- `--format <FORMAT>`: Output format. Options:
  - `human` (default, human-readable)
  - `raw` (exact byte counts)
  - `debug` (Rust struct dump)

---

## Ignore Semantics

- `.gitignore` in the root of the directory is automatically applied. Nested `.gitignore` files are ignored.  
- Ignore patterns are applied **during traversal**. Ignored directories are skipped entirely.  
- Negation patterns (`!`) cannot re-include files inside an ignored directory because the parent directory is not visited.  

Example:
```bash
fsx find /project --ignore "target/" --ignore "!target/keep/"  
```
In this example, `target/` is skipped entirely, so `target/keep/` is not visited even though it’s negated.

To selectively ignore contents but keep a subdirectory:
