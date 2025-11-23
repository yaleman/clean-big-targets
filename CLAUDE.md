# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`clean-big-targets` is a Rust CLI tool that finds and reports on Rust `target` directories (build artifacts) to help developers identify and optionally clean up disk space.

## Build & Development Commands

- `just check` - Run all checks (clippy, tests, and format verification) - **use this before considering any task complete**
- `just clippy` - Run clippy linter only
- `just test` - Run test suite
- `just fmt` - Verify code formatting (use `cargo fmt` to auto-format)
- `cargo run` - Run the tool (scans current directory by default)
- `cargo run -- --help` - See all CLI options

## Code Architecture

### Binary vs Library Split

The code is split between `src/main.rs` (binary) and `src/lib.rs` (library):

- **`src/main.rs`**: Entry point that handles CLI parsing, orchestrates the workflow, and uses rayon for parallel processing
- **`src/lib.rs`**: Contains all core functionality (directory scanning, size calculation, deletion handling)

This split allows the core logic to be unit tested and potentially reused by other tools.

### Key Components

1. **Directory Discovery** (`find_target_dirs`): Scans immediate child directories looking for `target` subdirectories. Special case: if the base directory itself is named "target", returns it immediately.

2. **Size Calculation** (`calculate_dir_size`): Recursively calculates total size of directories. Called in parallel using rayon for performance.

3. **Deletion Handler** (`handle_deletion`): Uses dialoguer's MultiSelect for interactive terminal prompts. Includes TTY detection to avoid errors in non-interactive contexts.

### Parallel Processing Strategy

The tool uses a two-phase approach:
1. **Sequential discovery**: `find_target_dirs` scans directories sequentially to build a list of paths
2. **Parallel sizing**: Rayon parallelizes the expensive size calculation across all discovered paths

This design avoids nested parallelism while maximizing performance on the slowest operation.

## Strict Lint Configuration

The codebase enforces strict linting (see `src/main.rs` lines 1-11):
- `#![deny(warnings)]` - All warnings are errors
- No `unwrap()`, `expect()`, `panic!()`, `todo!()`, or `unreachable!()` allowed
- Must handle errors explicitly with `Result` types
- `clippy::needless_pass_by_value` and `clippy::trivially_copy_pass_by_ref` enforced

When modifying code, always use `?` operator for error propagation and avoid any panic-inducing patterns.

## Adding Dependencies

Always use `cargo add <crate>` rather than manually editing `Cargo.toml` to ensure proper version resolution and feature selection.
