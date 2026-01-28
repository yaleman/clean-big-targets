use std::{
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
};

use clap::Parser;
use dialoguer::MultiSelect;
use humanize_bytes::humanize_bytes_decimal;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub debug: bool,
    #[clap(default_value = ".", env = "CLEAN_BIG_TARGETS_DIR")]
    pub target_dir: PathBuf,
    #[clap(short = 'D', long)]
    pub delete: bool,
    #[clap(long, requires = "delete")]
    pub force: bool,
}

#[derive(Debug)]
pub struct TargetDirInfo {
    pub path: PathBuf,
    pub size: u64,
}

pub fn find_target_dirs(base_dir: &Path, debug: bool) -> std::io::Result<Vec<PathBuf>> {
    let mut target_dirs = Vec::new();

    for entry in fs::read_dir(base_dir.canonicalize()?)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }
        if let Some(filename) = path.file_name()
            && filename == "target"
        {
            return Ok(vec![path]);
        }

        let target_path = path.join("target");
        if target_path.exists() && target_path.is_dir() {
            if debug {
                eprintln!("Found target directory: {:?}", target_path);
            }
            target_dirs.push(target_path);
        }
    }

    Ok(target_dirs)
}

pub fn calculate_dir_size(path: &PathBuf) -> std::io::Result<u64> {
    let mut total_size = 0u64;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                total_size += entry.metadata()?.len();
            } else if entry_path.is_dir() {
                total_size += calculate_dir_size(&entry_path)?;
            }
        }
    } else if path.is_file() {
        total_size = fs::metadata(path)?.len();
    }

    Ok(total_size)
}

pub fn handle_deletion(target_info: &[TargetDirInfo], force: bool) -> std::io::Result<()> {
    // Check if we can interact with the user

    if force {
        for info in target_info {
            match fs::remove_dir_all(&info.path) {
                Ok(_) => println!(
                    "Deleted '{}' successfully, ({})",
                    info.path.display(),
                    humanize_bytes_decimal!(info.size)
                ),
                Err(e) => {
                    eprintln!("Failed to delete: '{}' - giving up now!", e);
                    return Err(e);
                }
            }
        }
    } else {
        if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
            eprintln!("Cannot prompt for deletion: not running in interactive terminal");
            return Ok(());
        }
        println!("Prompting...");
        let items: Vec<String> = target_info
            .iter()
            .map(|info| {
                format!(
                    "{:>10}  {}",
                    humanize_bytes_decimal!(info.size),
                    info.path.display()
                )
            })
            .collect();

        let selections = MultiSelect::new()
            .with_prompt("Select target directories to delete (Space to select, Enter to confirm)")
            .items(&items)
            .interact()
            .map_err(std::io::Error::other)?;

        if selections.is_empty() {
            println!("No directories selected for deletion");
            return Ok(());
        }

        for &idx in &selections {
            let info = &target_info[idx];
            match fs::remove_dir_all(&info.path) {
                Ok(_) => println!(
                    "Deleted '{}' successfully, ({})",
                    info.path.display(),
                    humanize_bytes_decimal!(info.size)
                ),
                Err(e) => {
                    eprintln!("Failed to delete: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_dir_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let size = calculate_dir_size(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();

        let size = calculate_dir_size(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 13); // "Hello, World!" is 13 bytes
    }

    #[test]
    fn test_calculate_dir_size_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let mut file1 = File::create(temp_dir.path().join("file1.txt")).unwrap();
        file1.write_all(b"12345").unwrap();

        let mut file2 = File::create(temp_dir.path().join("file2.txt")).unwrap();
        file2.write_all(b"67890").unwrap();

        let size = calculate_dir_size(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();

        let mut file1 = File::create(temp_dir.path().join("top.txt")).unwrap();
        file1.write_all(b"abc").unwrap();

        let mut file2 = File::create(nested_dir.join("nested.txt")).unwrap();
        file2.write_all(b"defgh").unwrap();

        let size = calculate_dir_size(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 8);
    }

    #[test]
    fn test_find_target_dirs_none_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_target_dirs(temp_dir.path(), false).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_target_dirs_single() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project1");
        fs::create_dir(&project_dir).unwrap();
        fs::create_dir(project_dir.join("target")).unwrap();

        let result = find_target_dirs(temp_dir.path(), false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("project1/target"));
    }

    #[test]
    fn test_find_target_dirs_multiple() {
        let temp_dir = TempDir::new().unwrap();

        let project1 = temp_dir.path().join("project1");
        fs::create_dir(&project1).unwrap();
        fs::create_dir(project1.join("target")).unwrap();

        let project2 = temp_dir.path().join("project2");
        fs::create_dir(&project2).unwrap();
        fs::create_dir(project2.join("target")).unwrap();

        let project3 = temp_dir.path().join("project3");
        fs::create_dir(&project3).unwrap();

        let result = find_target_dirs(temp_dir.path(), false).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_target_dirs_child_is_target() {
        let temp_dir = TempDir::new().unwrap();

        // Create a child directory named "target"
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();

        // Scanning the parent should find the "target" directory and return it directly
        let result = find_target_dirs(temp_dir.path(), false).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("target"));
    }

    #[test]
    fn test_calculate_dir_size_on_package() {
        // This test runs calculate_dir_size on the package base directory
        // to verify it works on real directory structures
        let package_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap();

        let result = calculate_dir_size(&package_dir);
        assert!(
            result.is_ok(),
            "Should successfully calculate size of package directory"
        );

        let size = result.unwrap();
        assert!(size > 0, "Package directory should have non-zero size");

        // Print size for informational purposes (visible with --nocapture)
        eprintln!("Package directory size: {}", humanize_bytes_decimal!(size));
        assert!(humanize_bytes_decimal!(size).ends_with(" MB"))
    }
}
