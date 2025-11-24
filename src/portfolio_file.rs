use anyhow::{Context, Result};
use std::{fs, io::Write, path::PathBuf, time::SystemTime};

use crate::settings::Settings;
use std::rc::Rc;

pub fn path_from_name(name: &str, settings: Rc<Settings>) -> Result<PathBuf> {
    Ok(settings
        .portfolio_dir
        .clone()
        .join(name)
        .with_extension("csv"))
}

fn is_csv_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

// v4
fn list(settings: Rc<Settings>) -> Result<Vec<String>> {
    let path_str = &settings.portfolio_dir.clone();
    let entries = fs::read_dir(path_str)
        .with_context(|| format!("Failed to read directory: {:?}", path_str))?;

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

pub fn print_list(settings: Rc<Settings>) -> Result<()> {
    println!("\nFound csv files:\n");

    let l = list(settings)?;
    let size = l.len();
    if size == 0 {
        println!("no portfolios found");
    } else {
        println!("{}", l.join("\n"));
    }

    Ok(())
}

pub fn new(name: &str, settings: Rc<Settings>) -> Result<()> {
    let new_file_path: PathBuf = path_from_name(name, settings)?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&new_file_path)
        .with_context(|| format!("Error while creating csv file {:?}", &new_file_path))?;
    // write csv header
    file.write_all("created_at,pair,side,amount,price,fee\n".as_bytes())?;
    println!(
        "Successfully created new file: {}",
        new_file_path.to_str().unwrap_or("Unknown")
    );
    Ok(())
}
