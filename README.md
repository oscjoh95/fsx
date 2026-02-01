# fsx - File System Explorer

`fsx` is a small Rust CLI tool to explore directories, compute statistics, and search for files.

It can:

- Compute filesystem stats (files, directories, size, largest file, max depth)
- Search for files matching regex patterns
- Respect `.gitignore` rules in the root of the analyzed directory
- Optionally follow symlinks and limit recursion depth

---

## Installation

Build from source:
```bash
git clone https://github.com/oscjoh95/fsx.git  
cd fsx  
cargo build --release  
```
The binary will be available at `target/release/fsx`.

---

## Commands

### fsx stats

Compute filesystem statistics for a directory tree.

Basic usage:
```bash
fsx stats  
```
Analyze a specific directory:
```bash
fsx stats /path/to/dir  
```
For full options, ignore handling, and output formats, see [Stats Docs](docs/stats.md).

---

### fsx find

Search for files matching a regex pattern.

Basic usage (search current directory for all files):
```bash
fsx find --regex ".*"  
```
Search a specific directory:
```bash
fsx find /path/to/dir --regex ".*\.rs$"  
```
For full options, depth control, and advanced search, see [Find Docs](docs/find.md).

---

## Notes

- CLI ignore patterns are appended to `.gitignore` patterns and take precedence.  
- Currently, only `.gitignore` files in the root directory are supported.  
- Output formats: `human` (default), `raw` (exact bytes), `debug` (Rust struct dump).  
- For detailed usage examples and advanced options, see the docs in the `docs/` folder.