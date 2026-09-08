use super::PackageSpec;
use crate::hashing::sha256_prefixed;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

impl PackageSpec {
    pub fn canonicalized(&self) -> Self {
        let mut package = self.clone();
        package.files.sort();
        package.authors.sort();
        package.targets.sort();
        package.dependencies.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.constraint.cmp(&right.constraint))
                .then(left.optional.cmp(&right.optional))
        });
        package
    }

    pub fn tree_hash_for_root(&self, root: &std::path::Path) -> PrayResult<String> {
        let mut file_bytes = BTreeMap::new();
        for file in &self.files {
            let path = root.join(file);
            if !path.exists() {
                return Err(PrayError::Integrity(format!(
                    "package file missing: {file}"
                )));
            }
            if path.is_dir() {
                return Err(PrayError::Integrity(format!(
                    "package file is a directory: {file}"
                )));
            }
            file_bytes.insert(file.clone(), std::fs::read(&path)?);
        }
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
