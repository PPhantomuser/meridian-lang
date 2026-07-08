use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use tar::{Builder, Archive};
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
    pub checksum: String,
    pub dependencies: Option<HashMap<String, String>>, // simplified deps for index
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

pub fn get_cache_dir() -> PathBuf {
    let dir = get_registry_dir().join("cache");
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn get_src_dir() -> PathBuf {
    let dir = get_registry_dir().join("src");
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn publish_package(
    pkg_name: &str,
    version: &str,
    project_dir: &Path,
    deps: Option<HashMap<String, String>>,
) -> Result<(), String> {
    let cache_dir = get_cache_dir();
    let archive_path = cache_dir.join(format!("{}-{}.tar.gz", pkg_name, version));
    
    let tar_gz = File::create(&archive_path).map_err(|e| format!("Failed to create archive file: {}", e))?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    
    let toml_path = project_dir.join("meridian.toml");
    if toml_path.exists() {
        tar.append_path_with_name(&toml_path, "meridian.toml").map_err(|e| format!("Failed to add toml: {}", e))?;
    } else {
        return Err("meridian.toml not found".to_string());
    }
    
    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        tar.append_dir_all("src", &src_dir).map_err(|e| format!("Failed to add src dir: {}", e))?;
    } else {
        return Err("src/ directory not found".to_string());
    }
    
    tar.into_inner().map_err(|e| e.to_string())?.finish().map_err(|e| e.to_string())?;
    
    let mut file = File::open(&archive_path).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hex::encode(hasher.finalize());
    
    let index_dir = get_index_dir();
    let index_file = index_dir.join(format!("{}.json", pkg_name));
    
    let mut index = if index_file.exists() {
        let content = fs::read_to_string(&index_file).unwrap();
        serde_json::from_str(&content).unwrap_or(RegistryIndex {
            name: pkg_name.to_string(),
            versions: vec![],
        })
    } else {
        RegistryIndex {
            name: pkg_name.to_string(),
            versions: vec![],
        }
    };
    
    if index.versions.iter().any(|v| v.version == version) {
        return Err(format!("Version {} of package {} is already published.", version, pkg_name));
    }
    
    index.versions.push(RegistryVersion {
        version: version.to_string(),
        checksum: hash,
        dependencies: deps,
    });
    
    let updated_index = serde_json::to_string_pretty(&index).unwrap();
    fs::write(index_file, updated_index).unwrap();
    
    Ok(())
}

pub fn get_package_index(pkg_name: &str) -> Option<RegistryIndex> {
    let index_file = get_index_dir().join(format!("{}.json", pkg_name));
    if index_file.exists() {
        let content = fs::read_to_string(&index_file).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn extract_artifact(pkg_name: &str, version: &str, expected_checksum: &str) -> Result<PathBuf, String> {
    let archive_path = get_cache_dir().join(format!("{}-{}.tar.gz", pkg_name, version));
    if !archive_path.exists() {
        return Err(format!("Artifact not found in cache for {} {}", pkg_name, version));
    }
    
    let mut file = File::open(&archive_path).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hex::encode(hasher.finalize());
    
    if hash != expected_checksum {
        return Err(format!("Checksum mismatch. Expected {}, got {}", expected_checksum, hash));
    }
    
    let extract_dir = get_src_dir().join(format!("{}-{}", pkg_name, version));
    if !extract_dir.exists() {
        fs::create_dir_all(&extract_dir).unwrap();
        let tar_gz = File::open(&archive_path).unwrap();
        let dec = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(dec);
        archive.unpack(&extract_dir).map_err(|e| format!("Failed to unpack artifact: {}", e))?;
    }
    
    Ok(extract_dir)
}
