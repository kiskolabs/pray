use crate::paths::validate_package_relative_path;
use crate::resource_limits::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
};
use crate::{PrayError, PrayResult};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

pub(crate) fn unpack_praypkg(artifact_bytes: &[u8], output_directory: &Path) -> PrayResult<()> {
    if artifact_bytes.len() as u64 > MAX_ARCHIVE_TOTAL_BYTES {
        return Err(PrayError::Integrity(format!(
            "package archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )));
    }
    let cursor = std::io::Cursor::new(artifact_bytes);
    let decoder = zstd::stream::read::Decoder::new(cursor)
        .map_err(|error| PrayError::Integrity(error.to_string()))?;
    let mut archive = tar::Archive::new(decoder);
    let mut written_paths = BTreeSet::new();
    let mut total_bytes = 0u64;
    let mut entry_count = 0usize;

    for entry in archive
        .entries()
        .map_err(|error| PrayError::Integrity(error.to_string()))?
    {
        let mut entry = entry.map_err(|error| PrayError::Integrity(error.to_string()))?;
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        if entry_type.is_symlink() || entry_type.is_hard_link() || !entry_type.is_file() {
            return Err(PrayError::Integrity(
                "unsupported package archive entry type".to_string(),
            ));
        }
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(PrayError::Integrity(format!(
                "package archive exceeds {MAX_ARCHIVE_ENTRIES} entries"
            )));
        }
        let path = entry
            .path()
            .map_err(|error| PrayError::Integrity(error.to_string()))?
            .into_owned();
        validate_package_relative_path(&path)?;
        if !written_paths.insert(path.clone()) {
            return Err(PrayError::Integrity(format!(
                "duplicate package archive path: {}",
                path.display()
            )));
        }
        let size = entry.header().size().unwrap_or(0);
        if size > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "package archive entry exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes: {}",
                path.display()
            )));
        }
        total_bytes = total_bytes.saturating_add(size);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(PrayError::Integrity(format!(
                "package archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} decompressed bytes"
            )));
        }
        let destination = output_directory.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut destination_file = fs::File::create(&destination)?;
        let copied = std::io::copy(
            &mut (&mut entry).take(MAX_ARCHIVE_ENTRY_BYTES.saturating_add(1)),
            &mut destination_file,
        )
        .map_err(|error| PrayError::Integrity(error.to_string()))?;
        if copied > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "package archive entry exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tar::{Builder, Header};

    fn temporary_directory(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&path).expect("temp dir");
        path
    }

    fn pack_praypkg(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_bytes);
            for (path, contents) in entries {
                let mut header = Header::new_gnu();
                header.set_size(contents.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder
                    .append_data(&mut header, path, *contents)
                    .expect("append tar entry");
            }
            builder.finish().expect("finish tar");
        }
        compress_zstd(&tar_bytes)
    }

    fn pack_praypkg_with_raw_path(path: &str, contents: &[u8]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_bytes);
            let mut header = Header::new_gnu();
            let path_bytes = path.as_bytes();
            let gnu = header.as_gnu_mut().expect("gnu header");
            assert!(path_bytes.len() < gnu.name.len());
            gnu.name = [0; 100];
            gnu.name[..path_bytes.len()].copy_from_slice(path_bytes);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, contents).expect("append raw path");
            builder.finish().expect("finish tar");
        }
        compress_zstd(&tar_bytes)
    }

    fn compress_zstd(tar_bytes: &[u8]) -> Vec<u8> {
        let mut encoded = Vec::new();
        let mut encoder = zstd::stream::write::Encoder::new(&mut encoded, 0).expect("zstd");
        encoder.write_all(tar_bytes).expect("write zstd");
        encoder.finish().expect("finish zstd");
        encoded
    }

    #[test]
    fn unpack_praypkg_rejects_parent_directory_escape() {
        let artifact = pack_praypkg_with_raw_path("../escape.md", b"owned\n");
        let output = temporary_directory("pray-archive-escape");
        let error = unpack_praypkg(&artifact, &output).expect_err("escape");
        assert!(
            error.to_string().contains("escapes package root"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn unpack_praypkg_accepts_nested_relative_file() {
        let artifact = pack_praypkg(&[("exports/guidance.md", b"safe\n")]);
        let output = temporary_directory("pray-archive-ok");
        unpack_praypkg(&artifact, &output).expect("safe unpack");
        let text = fs::read_to_string(output.join("exports/guidance.md")).expect("read");
        assert_eq!(text, "safe\n");
    }

    #[test]
    fn unpack_praypkg_rejects_oversized_compressed_artifact() {
        let oversized = vec![0u8; (MAX_ARCHIVE_TOTAL_BYTES as usize) + 1];
        let output = temporary_directory("pray-archive-oversize");
        let error = unpack_praypkg(&oversized, &output).expect_err("oversize");
        assert!(
            error.to_string().contains("exceeds"),
            "unexpected error: {error}"
        );
    }
}
