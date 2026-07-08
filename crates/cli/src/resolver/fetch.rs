use std::path::{Path, PathBuf};
use std::process::Command;
use sha2::{Sha256, Digest};
use std::fs;

pub fn get_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".meridian").join("cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn fetch_git(url: &str, rev: Option<&str>) -> Result<(PathBuf, String), String> {
    let cache_dir = get_cache_dir();
    let git_db_dir = cache_dir.join("git").join("db");
    let git_checkout_dir = cache_dir.join("git").join("checkouts");
    
    fs::create_dir_all(&git_db_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&git_checkout_dir).map_err(|e| e.to_string())?;

    let url_hash = hash_url(url);
    let repo_dir = git_db_dir.join(&url_hash);

    if !repo_dir.exists() {
        let status = Command::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(url)
            .arg(&repo_dir)
            .status()
            .map_err(|e| e.to_string())?;
            
        if !status.success() {
            return Err(format!("Failed to clone {}", url));
        }
    } else {
        // Fetch updates
        let status = Command::new("git")
            .current_dir(&repo_dir)
            .arg("fetch")
            .arg("origin")
            .arg("+refs/heads/*:refs/heads/*")
            .status()
            .map_err(|e| e.to_string())?;
            
        if !status.success() {
            // Might be offline, continue with what we have
        }
    }

    let target_rev = rev.unwrap_or("HEAD");
    
    // Resolve rev to full commit hash to use as directory name and checksum
    let output = Command::new("git")
        .current_dir(&repo_dir)
        .arg("rev-parse")
        .arg(target_rev)
        .output()
        .map_err(|e| e.to_string())?;
        
    if !output.status.success() {
        return Err(format!("Revision {} not found in {}", target_rev, url));
    }
    
    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let checkout_dir = git_checkout_dir.join(&url_hash).join(&commit_hash);
    
    if !checkout_dir.exists() {
        fs::create_dir_all(&checkout_dir).map_err(|e| e.to_string())?;
        
        let status = Command::new("git")
            .current_dir(&repo_dir)
            .arg("--work-tree")
            .arg(&checkout_dir)
            .arg("checkout")
            .arg(&commit_hash)
            .arg("--")
            .arg(".")
            .status()
            .map_err(|e| e.to_string())?;
            
        if !status.success() {
            return Err(format!("Failed to checkout {}", commit_hash));
        }
    }
    
    Ok((checkout_dir, commit_hash))
}

pub fn compute_dir_checksum(dir: &Path) -> Result<String, String> {
    let mut paths = Vec::new();
    collect_files_recursive(dir, &mut paths).map_err(|e| e.to_string())?;
    paths.sort();

    let mut hasher = Sha256::new();
    for p in paths {
        // Skip .git directories
        if p.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        let rel = p.strip_prefix(dir).unwrap_or(&p);
        hasher.update(rel.to_string_lossy().as_bytes());
        let content = fs::read(&p).map_err(|e| e.to_string())?;
        hasher.update(&content);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

fn collect_files_recursive(dir: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, paths)?;
            } else {
                paths.push(path);
            }
        }
    }
    Ok(())
}
