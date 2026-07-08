pub mod manifest;
pub mod diagnostics;
pub mod fetch;
pub mod semver;
pub mod registry;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::resolver::manifest::{MeridianManifest, Lockfile, LockedPackage, Dependency};
use crate::resolver::diagnostics::ResolverDiagnostic;
use crate::resolver::fetch::{fetch_git, compute_dir_checksum};

pub struct ResolverResult {
    pub resolved_paths: HashMap<String, PathBuf>,
    pub diagnostics: Vec<ResolverDiagnostic>,
}

pub fn solve(
    workspace_root: &Path,
    manifest: &MeridianManifest,
    _offline: bool,
) -> ResolverResult {
    let mut resolved_paths = HashMap::new();
    let mut diagnostics = Vec::new();
    let mut locked_packages = Vec::new();

    // Read existing lockfile if any
    let lockfile_path = workspace_root.join("meridian.lock");
    let existing_lockfile: Option<Lockfile> = if lockfile_path.exists() {
        if let Ok(content) = fs::read_to_string(&lockfile_path) {
            toml::from_str(&content).ok()
        } else {
            None
        }
    } else {
        None
    };

    // Very basic resolution for MVP:
    // Just iterate over dependencies, fetch them if needed, and write new lockfile.
    if let Some(deps) = &manifest.dependencies {
        for (name, dep) in deps {
            match dep {
                Dependency::Path { path } => {
                    let full_path = workspace_root.join(path);
                    resolved_paths.insert(name.clone(), full_path);
                    
                    locked_packages.push(LockedPackage {
                        name: name.clone(),
                        version: "0.0.0".to_string(), // Path deps don't have strict versions here typically
                        source: format!("local+path+{}", path),
                        checksum: "".to_string(),
                        dependencies: vec![],
                    });
                }
                Dependency::Git { git, rev } => {
                    match fetch_git(git, rev.as_deref()) {
                        Ok((checkout_dir, commit_hash)) => {
                            let computed_checksum = compute_dir_checksum(&checkout_dir).unwrap_or_else(|_| commit_hash.clone());
                            
                            // Check against lockfile
                            if let Some(lockfile) = &existing_lockfile {
                                if let Some(locked_pkg) = lockfile.packages.iter().find(|p| p.name == *name) {
                                    if locked_pkg.checksum != computed_checksum {
                                        diagnostics.push(ResolverDiagnostic::ChecksumMismatch {
                                            package: name.clone(),
                                            expected: locked_pkg.checksum.clone(),
                                            actual: computed_checksum.clone(),
                                        });
                                        continue;
                                    }
                                }
                            }
                            
                            resolved_paths.insert(name.clone(), checkout_dir);
                            locked_packages.push(LockedPackage {
                                name: name.clone(),
                                version: "0.0.0".to_string(), // we might read the package's meridian.toml to get its actual version
                                source: format!("git+{}#{}", git, rev.as_deref().unwrap_or("HEAD")),
                                checksum: computed_checksum,
                                dependencies: vec![],
                            });
                        }
                        Err(e) => {
                            diagnostics.push(ResolverDiagnostic::FetchError {
                                package: name.clone(),
                                source: git.clone(),
                                error: e,
                            });
                        }
                    }
                }
                Dependency::Version(ver) => {
                    if let Some(index) = registry::get_package_index(name) {
                        let versions: Vec<String> = index.versions.iter().map(|v| v.version.clone()).collect();
                        if let Some(best_version_str) = semver::find_best_match(ver, versions.iter()) {
                            // Find the matching index entry for checksum
                            let reg_ver = index.versions.iter().find(|v| &v.version == best_version_str).unwrap();
                            let expected_checksum = &reg_ver.checksum;
                            
                            // Verify against lockfile if exists
                            let mut use_lockfile_version = false;
                            if let Some(lockfile) = &existing_lockfile {
                                if let Some(locked_pkg) = lockfile.packages.iter().find(|p| p.name == *name) {
                                    if locked_pkg.version == *best_version_str && locked_pkg.checksum == *expected_checksum {
                                        use_lockfile_version = true;
                                    } else {
                                        // Semver might have given a newer version, but if lockfile satisfies it, we should ideally use lockfile.
                                        // For MVP, we'll just check if lockfile version matches the requirement.
                                        if let Ok(req) = crate::resolver::semver::parse_version_req(ver) {
                                            if let Ok(v) = ::semver::Version::parse(&locked_pkg.version) {
                                                if req.matches(&v) {
                                                    // We can stick to lockfile version instead
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            match registry::extract_artifact(name, best_version_str, expected_checksum) {
                                Ok(extract_dir) => {
                                    resolved_paths.insert(name.clone(), extract_dir);
                                    locked_packages.push(LockedPackage {
                                        name: name.clone(),
                                        version: best_version_str.clone(),
                                        source: format!("registry+{}", name),
                                        checksum: expected_checksum.clone(),
                                        dependencies: vec![],
                                    });
                                }
                                Err(e) => {
                                    diagnostics.push(ResolverDiagnostic::FetchError {
                                        package: name.clone(),
                                        source: "registry".to_string(),
                                        error: format!("Failed to extract artifact: {}", e),
                                    });
                                }
                            }
                        } else {
                            diagnostics.push(ResolverDiagnostic::FetchError {
                                package: name.clone(),
                                source: "registry".to_string(),
                                error: format!("No matching version found for {} {} (available: {:?})", name, ver, versions),
                            });
                        }
                    } else {
                        diagnostics.push(ResolverDiagnostic::FetchError {
                            package: name.clone(),
                            source: "registry".to_string(),
                            error: format!("Package {} not found in local registry index", name),
                        });
                    }
                }
            }
        }
    }

    if diagnostics.is_empty() {
        let new_lockfile = Lockfile {
            version: 1,
            packages: locked_packages,
        };
        if let Ok(toml_str) = toml::to_string(&new_lockfile) {
            let _ = fs::write(lockfile_path, toml_str);
        }
    }

    ResolverResult {
        resolved_paths,
        diagnostics,
    }
}
