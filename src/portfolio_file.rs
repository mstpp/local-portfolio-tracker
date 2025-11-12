use anyhow::{Context, Result};
use std::{fs, path::PathBuf, time::SystemTime};

pub const PORTFOLIO_PATH: &str = "./portfolios";

pub fn path_from_name(name: &str) -> Result<PathBuf> {
    let path_buf = std::path::PathBuf::from(PORTFOLIO_PATH)
        .join(name)
        .with_extension("csv");
    Ok(path_buf)
}

pub fn path_str_from_name(name: &str) -> Result<String> {
    let path_buf = path_from_name(name)?;
    let path_str = path_buf.to_str().with_context(|| "Couldn't get path str")?;
    Ok(path_str.to_string())
}

// v1
// pub fn list() {
//     let paths = std::fs::read_dir("./portfolios").unwrap();
//     for path in paths {
//         println!("{}", path.unwrap().file_name().to_string_lossy());
//     }
// }

// v2 - refactoring after adding sorted list of files by creation time
// fn list() -> Result<Vec<String>> {
//     let entries = fs::read_dir(PORTFOLIO_PATH)
//         .with_context(|| format!("Failed to read directory: {}", PORTFOLIO_PATH))?;

//     let mut csv_files = Vec::new();
//     use std::collections::HashMap;
//     let mut csv_files_created: HashMap<String, SystemTime> = HashMap::new();

//     for entry in entries {
//         let entry = entry?;
//         let created_at: SystemTime = entry.metadata()?.created()?;
//         let path = entry.path();

//         // Skip non-files
//         if !path.is_file() {
//             continue;
//         }

//         // Only include .csv files (case-insensitive)
//         let is_csv = path
//             .extension()
//             .and_then(|ext| ext.to_str())
//             .map(|ext| ext.eq_ignore_ascii_case("csv"))
//             .unwrap_or(false);

//         if !is_csv {
//             continue;
//         }

//         // Extract filename stem
//         if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
//             csv_files.push(stem.to_string());
//             csv_files_created.insert(stem.to_string(), created_at);
//         }
//     }

//     let mut sorted_hm: Vec<_> = csv_files_created.iter().collect();
//     sorted_hm.sort_by_key(|&(_, time)| time);
//     let sorted_path: Vec<String> = sorted_hm.iter().map(|(p, _)| p.to_string()).collect();

//     // Ok(csv_files)
//     Ok(sorted_path)
// }

// v3 unreadable - never used
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

fn is_csv_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

// v4
fn list() -> Result<Vec<String>> {
    let entries = fs::read_dir(PORTFOLIO_PATH)
        .with_context(|| format!("Failed to read directory: {}", PORTFOLIO_PATH))?;

    let mut csv_files: Vec<(String, SystemTime)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() || !is_csv_file(&path) {
                return None;
            }

            let stem = path.file_stem()?.to_str()?.to_string();
            let created_at = entry.metadata().ok()?.created().ok()?;

            Some((stem, created_at))
        })
        .collect();

    // Sort by creation time
    csv_files.sort_by_key(|(_, time)| *time);

    Ok(csv_files.into_iter().map(|(name, _)| name).collect())
}

pub fn print_list() -> Result<()> {
    println!("\nFound csv files:\n");

    for item in list()? {
        println!("\t{}", item);
    }

    Ok(())
}

pub fn new(name: &str) -> Result<()> {
    // v1 - use PathBuf instead
    // let path = format!("./portfolios/{}", name);
    let new_file_path = path_from_name(name)?;
    // v1: if name exists, it will overwrite the existing file
    std::fs::File::create(new_file_path.clone()).with_context(|| "Couldn't create new csv file")?;
    println!(
        "Successfully created new file: {}",
        new_file_path.to_str().unwrap_or("Unknown")
    );
    // v2: TODO if exists, propt for action or check for --force cli param flat
    Ok(())
}
