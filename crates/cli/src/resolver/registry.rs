use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryIndex {
    pub name: String,
    pub versions: Vec<RegistryVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryVersion {
    pub version: String,
    pub source: String, // e.g., git repository URL
    pub checksum: String, // commit hash or similar checksum
    pub dependencies: Option<HashMap<String, String>>,
}

pub fn get_registry_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let reg_dir = PathBuf::from(home).join(".meridian").join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    reg_dir
}

pub fn get_index_dir() -> PathBuf {
    let dir = get_registry_dir().join("index");
    fs::create_dir_all(&dir).unwrap();
    dir
}

// Generates the JSON file that a user should submit to the central registry.
pub fn generate_publish_payload(
    pkg_name: &str,
    version: &str,
    source: &str,
    commit_hash: &str,
    deps: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let mut index = RegistryIndex {
        name: pkg_name.to_string(),
        versions: vec![],
    };
    
    // We only generate for this specific version to show the user what to append or create
    index.versions.push(RegistryVersion {
        version: version.to_string(),
        source: source.to_string(),
        checksum: commit_hash.to_string(),
        dependencies: deps,
    });
    
    serde_json::to_string_pretty(&index).map_err(|e| e.to_string())
}

pub fn get_package_index(pkg_name: &str) -> Option<RegistryIndex> {
    let index_file = get_index_dir().join(format!("{}.json", pkg_name));
    
    // Fetch from meridian-lang/registry using GitHub raw content API
    let registry_base = std::env::var("MERIDIAN_REGISTRY_URL")
        .unwrap_or_else(|_| "https://raw.githubusercontent.com/meridian-lang/registry/main".to_string());
    let url = format!("{}/index/{}.json", registry_base, pkg_name);
    
    if let Ok(response) = ureq::get(&url).call() {
        if let Ok(content) = response.into_string() {
            if let Ok(index) = serde_json::from_str::<RegistryIndex>(&content) {
                // Cache it locally
                let _ = fs::write(&index_file, &content);
                return Some(index);
            }
        }
    }
    
    // Fallback to cache if network fails or package not found online
    if index_file.exists() {
        let content = fs::read_to_string(&index_file).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}
