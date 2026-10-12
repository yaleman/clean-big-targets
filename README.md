# clean-big-targets

Find Rust build artifacts taking up disk space and optionally delete them. The
default command lists discovered `target` directories from largest to smallest,
with human-readable sizes and a total. Nothing is deleted unless you pass
`--delete`.

## Installation

With Rust and Cargo installed, install from GitHub:

```sh
cargo install --git https://github.com/yaleman/clean-big-targets.git --locked
```

Or install from a local checkout:

```sh
cargo install --path . --locked
```

Make sure Cargo's binary directory (usually `$HOME/.cargo/bin`) is on your `PATH`.

## Usage

List build artifacts in the current directory:

```sh
clean-big-targets
```

Scan a directory containing your Rust projects:

```sh
clean-big-targets ~/Projects
```

For a single project, pass the project directory:

```sh
clean-big-targets ~/Projects/my-project
```

### What gets scanned

The scan looks for `target` directories directly inside immediate child
directories. For example, scanning `~/Projects` finds
`~/Projects/my-project/target`, but does not search deeper for
`~/Projects/group/my-project/target`.

If the scanned directory contains a direct child named `target`, the scan returns
only that directory. Pass the project directory, rather than the `target`
directory itself. Custom Cargo build directories with other names are not
discovered.

Sizes include files recursively inside each discovered directory. They represent
file sizes, so the reported total may differ from the disk space actually freed.

### Delete build artifacts

Choose directories to delete in an interactive terminal:

```sh
clean-big-targets ~/Projects --delete
```

Use the arrow keys to move, **Space** to select directories, and **Enter** to
confirm and delete the selection. Confirming with nothing selected deletes
nothing. Without an interactive terminal, this command prints a message and
skips deletion.

To delete every discovered directory without a prompt:

```sh
clean-big-targets ~/Projects --delete --force
```

Deletion removes the entire selected directory and its contents. Review the
listing first; `--force` skips selection and confirmation. Rust build artifacts
will need to be rebuilt the next time you build those projects.

### Default scan directory

Set `CLEAN_BIG_TARGETS_DIR` to use a default directory instead of the current
directory:

```sh
export CLEAN_BIG_TARGETS_DIR="$HOME/Projects"
clean-big-targets
```

An explicit directory argument overrides this environment variable.

### Options

```text
clean-big-targets [OPTIONS] [TARGET_DIR]
```

| Argument or option | Description |
| --- | --- |
| `[TARGET_DIR]` | Directory to scan; defaults to `CLEAN_BIG_TARGETS_DIR`, or `.` if unset. |
| `-D`, `--delete` | Interactively select directories to delete. |
| `--force` | Delete all discovered directories without prompting; requires `--delete`. |
| `-d`, `--debug` | Print diagnostic information. |
| `-h`, `--help` | Show command help. |

If no directories are found, the command reports that and exits successfully.
Errors reading a discovered directory's size are reported, and that directory
is skipped.
