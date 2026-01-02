# EMBARGO

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

Fast codebase dependency extraction optimized for AI code analysis.

EMBARGO analyzes source code and generates structured dependency graphs that large language models can easily understand. It processes multiple programming languages and outputs compact, information-dense formats perfect for AI-assisted development.

## Features

- **Fast processing** - Analyze thousands of lines per second
- **AI-ready output** - Compact format with inline function signatures
- **Multi-language** - Supports 8+ programming languages
- **Parallel execution** - Efficient memory usage
- **Zero config** - Automatic language detection

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/lrnzgiusti/embargo.git
cd embargo

# Build from source
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Basic Usage

```bash
# Analyze current directory
embargo .

# Analyze specific directory
embargo /path/to/project

# Output to custom file
embargo --output analysis.md /path/to/project

# Use LLM-optimized format (compact, inline signatures)
embargo --format llm-optimized /path/to/project

# JSON output format
embargo --format json-compact /path/to/project

# Analyze specific languages only
embargo --languages python,typescript /path/to/project

# Include specific files
embargo --include "src/**/*.rs" /path/to/project
```

## Output Format

EMBARGO generates analysis files with function signatures and dependency information. The LLM-optimized format groups code by architecture and shows relationships between functions:

```markdown
### UTILITY_LAYER
NODES:566 CALL_DEPTH:1

analyzer.rs→[new(())[ENTRY],analyze((&mut self, root_path: &Path, languages: &[&str]))] 
cache.rs→[get((&self, file_path: &Path))[HOT]→{load_from_disk,store_to_disk}]
```

- `[ENTRY]` marks public API entry points
- `[HOT]` identifies performance-critical functions  
- `→{calls}` shows function dependencies
- Full parameter types included inline for better AI understanding

## Use Cases

- **AI code review** - Feed complete codebase context to language models
- **Architecture analysis** - Understand dependencies and relationships  
- **Code navigation** - Quick overview of large projects
- **Documentation** - Auto-generate API docs with dependency info

## Performance

Fast analysis through parallel processing and efficient parsing:


```
$ embargo -i src/ -o ./embargo_compact.md -l rust -f json-compact
EMBARGO - Codebase Analysis
Input: src/
Output: ./embargo_compact.md
Format: json-compact
Languages: ["rust"]
Scanning files...
Found 21 files to analyze
Parsing files with cache optimization...
Cache hits: 21, Parsed: 0
Building dependency graph...
Resolving function calls...
Skipped call resolution for maximum performance
Analysis completed in 0.01s
JSON output: ./embargo_compact.json
Analysis complete! Generated ./embargo_compact.md
Total execution time: 0.01s
```

```
$ embargo -i src/ -o ./embargo.md -l rust -f markdown
EMBARGO - Codebase Analysis
Input: src/
Output: ./embargo.md
Format: markdown
Languages: ["rust"]
Scanning files...
Found 21 files to analyze
Parsing files with cache optimization...
Cache hits: 21, Parsed: 0
Building dependency graph...
Resolving function calls...
Skipped call resolution for maximum performance
Analysis completed in 0.01s
Analysis complete! Generated ./embargo.md
Total execution time: 0.01s
```

```
$ embargo -i src/ -o ./embargo_llm_opt.md -l rust -f llm-optimized
EMBARGO - Codebase Analysis
Input: src/
Output: ./embargo_llm_opt.md
Format: llm-optimized
Languages: ["rust"]
Scanning files...
Found 21 files to analyze
Parsing files with cache optimization...
Cache hits: 21, Parsed: 0
Building dependency graph...
Resolving function calls...
Skipped call resolution for maximum performance
Analysis completed in 0.02s
Analysis complete! Generated ./embargo_llm_opt.md
Total execution time: 0.04s
```
## Supported Languages

Python, TypeScript, Rust, C++, JavaScript, Java, C#, Go

Each language parser extracts:
- Function/method definitions with full signatures
- Class/struct declarations and relationships  
- Import/dependency statements
- Call sites and usage patterns

## Development

```bash
cargo test          # Run tests
cargo run -- .      # Analyze current directory  
cargo fmt           # Format code
cargo clippy        # Lint
```

### Testing

```bash
# Run the full test suite
cargo test

# Run only unit tests (skip benches/examples)
cargo test --lib

# Filter by test name (substring match)
cargo test graph_builder

# Parser tests by language
cargo test parser_rust
cargo test parser_python
cargo test parser_typescript

# Analyzer end-to-end tests
cargo test analyzer_end_to_end_on_small_rust_file
cargo test analyzer_on_python_app_directory
cargo test analyzer_on_typescript_app_directory

# Formatter snapshot/golden tests (stable, deterministic)
cargo test llm_formatter_files_and_directory_tree_are_stable
cargo test llm_clusters_large_graph_matches_golden
cargo test json_compact_snapshot_small_graph
```

Notes:
- Snapshot tests are deterministic: the LLM-optimized formatter output is sorted and stable.
- Golden file for clustered LLM view lives at `tests/golden/llm_clusters_large.md`.
- Integration tests use fixtures under `test_apps/`.

## License

Licensed under the Apache License, Version 2.0.
