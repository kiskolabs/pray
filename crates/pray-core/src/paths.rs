use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Repository-relative path that cannot escape a project root when joined.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectRelativePath(PathBuf);

impl ProjectRelativePath {
    pub fn parse(value: &str) -> PrayResult<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(PrayError::Manifest(
                "project path must not be empty".to_string(),
            ));
        }
        let path = Path::new(trimmed);
        if path.is_absolute() {
            return Err(PrayError::Manifest(format!(
                "project path must be repository-relative: {trimmed}"
            )));
        }
        let mut relative = PathBuf::new();
        for component in path.components() {
            match component {
                Component::Normal(part) => relative.push(part),
                Component::CurDir => {}
                _ => {
                    return Err(PrayError::Manifest(format!(
                        "project path escapes repository root: {trimmed}"
                    )));
                }
            }
        }
        if relative.as_os_str().is_empty() {
            return Err(PrayError::Manifest(format!(
                "project path must be repository-relative: {trimmed}"
            )));
        }
        Ok(Self(relative))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap_or("")
    }

    pub fn join_root(&self, root: &Path) -> PathBuf {
        root.join(&self.0)
    }
}

pub fn validate_project_relative_path(value: &str) -> PrayResult<ProjectRelativePath> {
    ProjectRelativePath::parse(value)
}

pub fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
    let mut prayspec_files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("prayspec") {
            prayspec_files.push(path);
        }
    }
    match prayspec_files.len() {
        1 => Ok(prayspec_files.remove(0)),
        0 => Err(PrayError::Resolution(format!(
            "no prayspec file found in {:?}",
            root
        ))),
        _ => Err(PrayError::Resolution(format!(
            "multiple prayspec files found in {:?}",
            root
        ))),
    }
}

pub fn validate_package_relative_path(path: &Path) -> PrayResult<()> {
    if path.is_absolute() {
        return Err(PrayError::Integrity(format!(
            "package path must be relative: {}",
            path.display()
        )));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => {
                return Err(PrayError::Integrity(format!(
                    "package path escapes package root: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

pub fn sanitize_relative_path(path: &str) -> PrayResult<PathBuf> {
    let path = path.trim_start_matches('/');
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            _ => {
                return Err(PrayError::Resolution(format!(
                    "invalid relative path: {path}"
                )));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(PrayError::Resolution(format!(
            "invalid relative path: {path}"
        )));
    }
    Ok(relative)
}

pub fn remove_path_if_exists(path: &Path) -> PrayResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
            Ok(())
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_relative_path_rejects_parent_dir() {
        let error = sanitize_relative_path("../escape.praypkg").expect_err("parent dir");
        assert!(error.to_string().contains("invalid relative path"));
    }

    #[test]
    fn sanitize_relative_path_accepts_nested_artifact() {
        let path = sanitize_relative_path("v1/artifacts/sample/base/1.0.0/package.praypkg")
            .expect("nested path");
        assert_eq!(
            path,
            PathBuf::from("v1/artifacts/sample/base/1.0.0/package.praypkg")
        );
    }

    #[test]
    fn validate_package_relative_path_rejects_parent_escape() {
        let error =
            validate_package_relative_path(Path::new("../escape.md")).expect_err("parent escape");
        assert!(error.to_string().contains("escapes package root"));
    }

    #[test]
    fn validate_package_relative_path_rejects_absolute() {
        let absolute = if cfg!(windows) {
            Path::new(r"C:\escape.md")
        } else {
            Path::new("/escape.md")
        };
        let error = validate_package_relative_path(absolute).expect_err("absolute");
        assert!(error.to_string().contains("must be relative"));
    }

    #[test]
    fn validate_package_relative_path_accepts_nested_file() {
        validate_package_relative_path(Path::new("exports/guidance.md")).expect("nested file");
    }

    #[test]
    fn project_relative_path_rejects_parent_and_absolute() {
        let parent = ProjectRelativePath::parse("../escape.md").expect_err("parent");
        assert!(parent.to_string().contains("escapes repository root"));
        let absolute = if cfg!(windows) {
            ProjectRelativePath::parse(r"C:\escape.md")
        } else {
            ProjectRelativePath::parse("/tmp/escape.md")
        }
        .expect_err("absolute");
        assert!(absolute.to_string().contains("repository-relative"));
    }

    #[test]
    fn project_relative_path_joins_under_root() {
        let relative = ProjectRelativePath::parse("docs/AGENTS.md").expect("relative");
        assert_eq!(
            relative.join_root(Path::new("/repo")),
            PathBuf::from("/repo/docs/AGENTS.md")
        );
    }
}
