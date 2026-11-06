use super::PackageSpec;
use crate::hashing::prefixed_hex_digest;
use crate::PrayResult;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

impl PackageSpec {
    pub fn into_canonicalized(mut self) -> Self {
        self.files.sort();
        self.authors.sort();
        self.maintainers.sort();
        self.targets.sort();
        self.related = crate::package_relation::canonicalize(&self.related);
        self.dependencies.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.constraint.cmp(&right.constraint))
                .then(left.optional.cmp(&right.optional))
        });
        self
    }

    pub fn canonicalized(&self) -> Self {
        self.clone().into_canonicalized()
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
        // BTreeMap keys are already sorted; stream the same bytes the prior
        // string builder emitted so the digest stays identical.
        let mut hasher = Sha256::new();
        for (path, bytes) in file_bytes {
            hasher.update(b"file\0regular\0");
            hasher.update(path.as_bytes());
            hasher.update(b"\0");
            hasher.update(b"sha256:");
            update_hex(&mut hasher, Sha256::digest(bytes));
            hasher.update(b"\n");
        }
        Ok(prefixed_hex_digest(hasher.finalize()))
    }
}

fn update_hex(hasher: &mut Sha256, digest: impl AsRef<[u8]>) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest.as_ref() {
        hasher.update([HEX[(byte >> 4) as usize], HEX[(byte & 0xf) as usize]]);
    }
}

#[cfg(test)]
mod tests {
    use super::PackageSpec;
    use crate::hashing::{prefixed_hex_digest, sha256_prefixed};
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    #[test]
    fn tree_hash_matches_legacy_string_builder() {
        let mut file_bytes = BTreeMap::new();
        file_bytes.insert("b.md".to_string(), b"beta".to_vec());
        file_bytes.insert("a.md".to_string(), b"alpha".to_vec());

        let mut serialized = String::new();
        for (path, bytes) in &file_bytes {
            serialized.push_str("file\0regular\0");
            serialized.push_str(path);
            serialized.push('\0');
            serialized.push_str(&sha256_prefixed(bytes));
            serialized.push('\n');
        }
        let legacy = sha256_prefixed(serialized.as_bytes());
        let streamed = PackageSpec::tree_hash_from_file_bytes(&file_bytes).expect("hash");
        assert_eq!(streamed, legacy);
        assert_eq!(
            streamed,
            prefixed_hex_digest({
                let mut hasher = Sha256::new();
                hasher.update(serialized.as_bytes());
                hasher.finalize()
            })
        );
    }
}
