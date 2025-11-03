use anyhow::{Context, Result};
use std::fs;

const PORTFOLIO_PATH: &str = "./portfolios";

fn list() -> Result<Vec<String>> {
    let entries = fs::read_dir(PORTFOLIO_PATH)
        .with_context(|| format!("Failed to read directory: {}", PORTFOLIO_PATH))?;

    let mut csv_files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip non-files
        if !path.is_file() {
            continue;
        }

        // Only include .csv files (case-insensitive)
        let is_csv = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            .unwrap_or(false);

        if !is_csv {
            continue;
        }

        // Extract filename stem
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            csv_files.push(stem.to_string());
        }
    }

    Ok(csv_files)
}

pub fn print_list() -> Result<()> {
    println!("\nFound csv files:\n");

    for item in list()? {
        println!("\t{}", item);
    }

    Ok(())
}

// v1
// /// List all files in ./portfolios dir
// pub fn list() {
//     let paths = std::fs::read_dir("./portfolios").unwrap();
//     for path in paths {
//         println!("{}", path.unwrap().file_name().to_string_lossy());
//     }
// }

// v3 unreadable
// pub fn list() -> Result<Vec<String>> {
//     let csv_files = fs::read_dir(PORTFOLIO_PATH)
//         .with_context(|| format!("Failed to read directory: {}", PORTFOLIO_PATH))?
//         .filter_map(|entry| entry.ok())
//         .map(|entry| entry.path())
//         .filter(|path| {
//             path.is_file()
//                 && path
//                     .extension()
//                     .and_then(|ext| ext.to_str())
//                     .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
//         })
//         .filter_map(|path| {
//             path.file_stem()
//                 .and_then(|s| s.to_str())
//                 .map(String::from)
//         })
//         .collect();
//     Ok(csv_files)
// }

pub fn new(name: &str) -> Result<()> {
    // v1 - use PathBuf instead
    // let path = format!("./portfolios/{}", name);
    let new_file_path = std::path::PathBuf::from(PORTFOLIO_PATH)
        .join(name)
        .with_extension("csv");
    // v1: if name exists, it will overwrite the existing file
    std::fs::File::create(new_file_path.clone()).with_context(|| "Couldn't create new csv file")?;
    println!(
        "Successfully created new file: {}",
        new_file_path.to_str().unwrap_or("Unknown")
    );
    // v2: TODO if exists, propt for action or check for --force cli param flat
    Ok(())
}
