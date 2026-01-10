# fsx - File System Explorer

`fsx` is a small Rust tool to explore directories and report filesystem statistics.  

It recursively walks a directory tree and computes:

- Total files
- Total directories
- Total size
- Largest file
- Maximum depth

It continues on errors, reporting unreadable files or directories as warnings.

---

## Installation

Build from source with Rust:

```bash
git clone https://github.com/<your-username>/fsx.git
cd fsx
cargo build --release
``` 
The binary will be at target/release/fsx.

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

### Output formats
```bash 
 fsx stats --format human # default, human-readable sizes 
 fsx stats --format raw # exact byte counts 
 fsx stats --format debug # Rust struct dump 
 ```

## Options & Notes
### Arguments
```bash
[PATH] Root directory to analyze Default: . 
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
```

### Examples
```bash 
# Analyze current directory with default human-readable output 
fsx stats

# Analyze specific directory with raw output
fsx stats /path/to/dir --format raw

# Limit depth to 2
fsx stats /path/to/dir --max-depth 2
```