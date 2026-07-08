use semver::{Version, VersionReq};

// MVP semver resolution.
// In the future this will implement a backtracking dependency graph solver.
pub fn parse_version_req(req: &str) -> Result<VersionReq, String> {
    VersionReq::parse(req).map_err(|e| e.to_string())
}

pub fn find_best_match<'a>(req_str: &str, available_versions: impl Iterator<Item = &'a String>) -> Option<&'a String> {
    let req = parse_version_req(req_str).ok()?;
    let mut best_match: Option<(&String, Version)> = None;
    
    for v_str in available_versions {
        if let Ok(v) = Version::parse(v_str) {
            if req.matches(&v) {
                if let Some((_, best_v)) = &best_match {
                    if v > *best_v {
                        best_match = Some((v_str, v));
                    }
                } else {
                    best_match = Some((v_str, v));
                }
            }
        }
    }
    
    best_match.map(|(v_str, _)| v_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_req() {
        assert!(parse_version_req("^1.2.3").is_ok());
        assert!(parse_version_req("~1.2").is_ok());
        assert!(parse_version_req("=1.0.0").is_ok());
        assert!(parse_version_req("invalid_version").is_err());
    }
}
