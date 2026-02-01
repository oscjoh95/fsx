# fsx stats

The `fsx stats` command computes filesystem statistics for a directory tree.

It recursively walks the directory and reports file and directory counts, total size, largest file, and maximum depth reached during traversal. Errors (e.g., unreadable files) are reported as warnings but do not stop the analysis.

---

## Basic Usage

Analyze the current directory:
```bash
fsx stats  
```
Analyze a specific directory:
```bash
fsx stats /path/to/dir  
```
---

## Options

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
fsx stats /project --ignore "target/" --ignore "!target/keep/"  
```
In this case, `target/` is skipped entirely. `target/keep/` is not visited even though it’s negated.

To selectively ignore contents but keep a subdirectory:
```bash
--ignore "target/*" --ignore "!target/keep/"
```
---

## Output

The command prints the following statistics:

- Total files  
- Total directories  
- Total symlinks  
- Total size  
- Largest file  
- Maximum depth reached

Output respects the `--format` option (`human`, `raw`, `debug`).

---

## Examples

- Analyze current directory with default human-readable output:
```bash
fsx stats
```
- Analyze a specific directory with raw output:
```bash
fsx stats /path/to/dir --format raw
```
- Limit recursion depth to 2:
```bash
fsx stats /path/to/dir --max-depth 2
```
- Follow symlinks but ignore build artifacts:
```bash
fsx stats --follow-symlinks --ignore "target/" --ignore "*.log"
```