use std::{env, fs, path::PathBuf};

pub const NUOP_SCRIPT_PATH: &str = "NUOP_SCRIPT_PATH";
pub const NUOP_MAPPINGS_PATH: &str = "NUOP_MAPPINGS_PATH";

pub fn get_script_path() -> String {
    env::var(NUOP_SCRIPT_PATH).unwrap_or_else(|_| "/scripts".to_string())
}

pub fn get_mapping_path() -> String {
    env::var(NUOP_MAPPINGS_PATH).unwrap_or_else(|_| "/config/mappings".to_string())
}

pub fn find_mappings(mappings_path: &str) -> Vec<PathBuf> {
    let mut mapping_files = Vec::new();
    let mappings_path = PathBuf::from(mappings_path);
    if mappings_path.is_dir() {
        for entry in fs::read_dir(mappings_path).expect("Failed to read directory {mappings_path}")
        {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                mapping_files.extend(find_mappings(path.to_str().unwrap()));
            } else if path.extension() == Some("yaml".as_ref()) {
                mapping_files.push(path);
            }
        }
    }

    mapping_files
}

pub fn find_scripts(script_path: &str) -> Vec<PathBuf> {
    let mut main_files = Vec::new();
    let script_path = PathBuf::from(script_path);

    if script_path.is_dir() {
        for entry in fs::read_dir(script_path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.is_dir() {
                // Check if this directory contains a mod.nu file
                let mod_nu_path = path.join("mod.nu");
                if mod_nu_path.exists() && mod_nu_path.is_file() {
                    main_files.push(mod_nu_path);
                } else {
                    // Recurse into subdirectories that don't have mod.nu
                    main_files.extend(find_scripts(path.to_str().unwrap()));
                }
            }
        }
    }

    main_files
}
