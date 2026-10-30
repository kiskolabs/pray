use super::PackageSpec;
use crate::hashing::sha256_prefixed;
use crate::PrayResult;
use std::collections::BTreeMap;

impl PackageSpec {
    pub fn canonicalized(&self) -> Self {
        let mut package = self.clone();
        package.files.sort();
        package.authors.sort();
        package.maintainers.sort();
        package.targets.sort();
        package.related = crate::package_relation::canonicalize(&package.related);
        package.dependencies.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.constraint.cmp(&right.constraint))
                .then(left.optional.cmp(&right.optional))
        });
        package
    }

    pub fn validate_relative_paths(&self) -> PrayResult<()> {
        use crate::paths::validate_package_relative_path;
        use std::path::Path;
        for file in &self.files {
            validate_package_relative_path(Path::new(file))?;
        }
        for export in self.exports.values() {
            validate_package_relative_path(Path::new(&export.path))?;
            if let Some(default_path) = &export.default_path {
                validate_package_relative_path(Path::new(default_path))?;
            }
        }
        for skill in self.skills.values() {
            validate_package_relative_path(Path::new(&skill.path))?;
        }
        for template in self.templates.values() {
            validate_package_relative_path(Path::new(&template.path))?;
        }
        Ok(())
    }

    pub fn tree_hash_for_root(&self, root: &std::path::Path) -> PrayResult<String> {
        self.validate_relative_paths()?;
        let file_bytes = crate::resolve_exports::load_package_file_bytes(root, self)?;
        Self::tree_hash_from_file_bytes(&file_bytes)
    }

    pub fn tree_hash_from_file_bytes(file_bytes: &BTreeMap<String, Vec<u8>>) -> PrayResult<String> {
        let mut entries = file_bytes
            .iter()
            .map(|(path, bytes)| (path.clone(), sha256_prefixed(bytes)))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut serialized = String::new();
        for (path, hash) in entries {
            serialized.push_str("file\0regular\0");
            serialized.push_str(&path);
            serialized.push('\0');
            serialized.push_str(&hash);
            serialized.push('\n');
        }
        Ok(sha256_prefixed(serialized.as_bytes()))
    }
}
