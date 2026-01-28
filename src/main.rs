#![deny(warnings)]
#![warn(unused_extern_crates)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]

use std::process::ExitCode;

use clap::Parser;
use clean_big_targets::{
    Cli, TargetDirInfo, calculate_dir_size, find_target_dirs, format_size, handle_deletion,
};
use rayon::prelude::*;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.debug {
        eprintln!("Debug mode is on");
    }

    if !cli.target_dir.exists() {
        eprintln!("Target directory does not exist: {:?}", cli.target_dir);
        return ExitCode::FAILURE;
    }

    if cli.debug {
        eprintln!("Target directory: {:?}", cli.target_dir);
    }

    // Find all target directories
    let target_dirs = match find_target_dirs(&cli.target_dir, cli.debug) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("Error scanning directories: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if target_dirs.is_empty() {
        eprintln!("No target directories found");
        return ExitCode::SUCCESS;
    }

    if cli.debug {
        eprintln!("Found {} target directories", target_dirs.len());
    }

    // Calculate sizes in parallel using rayon
    let mut target_info: Vec<TargetDirInfo> = target_dirs
        .par_iter()
        .filter_map(|path| match calculate_dir_size(path) {
            Ok(size) => Some(TargetDirInfo {
                path: path.clone(),
                size,
            }),
            Err(e) => {
                eprintln!("Error calculating size for {:?}: {}", path, e);
                None
            }
        })
        .collect();

    // Sort by size (largest first)
    target_info.sort_by(|a, b| b.size.cmp(&a.size));

    // Display results
    if !cli.delete {
        println!("\nTarget directories (sorted by size):");
        println!("{:>10}  PATH", "SIZE");
        println!("{}", "-".repeat(80));
        for info in &target_info {
            println!("{:>10}  {}", format_size(info.size), info.path.display());
        }
        let total_size: u64 = target_info.iter().map(|i| i.size).sum();
        println!("{}", "-".repeat(80));
        println!("{:>10}  Total", format_size(total_size));
    } else if let Err(e) = handle_deletion(&target_info, cli.force) {
        eprintln!("Error during deletion: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
