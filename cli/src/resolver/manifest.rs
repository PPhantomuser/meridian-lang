use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeridianManifest {
    pub package: Package,
    pub dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Path { path: String },
    Git { git: String, rev: Option<String> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lockfile {
    pub version: u32,
    #[serde(rename = "package", default)]
    pub packages: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
            [package]
            name = "test_pkg"
            version = "1.0.0"

            [dependencies]
            local_dep = { path = "../local" }
            git_dep = { git = "https://github.com/test/test", rev = "abcdef" }
            ver_dep = "1.2.3"
        "#;

        let manifest: MeridianManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "test_pkg");
        
        let deps = manifest.dependencies.unwrap();
        assert_eq!(deps.len(), 3);
        
        match &deps["git_dep"] {
            Dependency::Git { git, rev } => {
                assert_eq!(git, "https://github.com/test/test");
                assert_eq!(rev.as_ref().unwrap(), "abcdef");
            }
            _ => panic!("Expected Git dependency"),
        }
    }
}
