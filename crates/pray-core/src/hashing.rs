use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

pub fn sha256_prefixed(bytes: &[u8]) -> String {
    prefixed_hex_digest(Sha256::digest(bytes))
}

fn prefixed_hex_digest(digest: impl AsRef<[u8]>) -> String {
    let digest = digest.as_ref();
    let mut output = String::with_capacity(7 + digest.len() * 2);
    output.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

pub fn marker_id(seed: &str) -> String {
    sha256_hex(seed.as_bytes())[0..8].to_string()
}

pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn checksum_managed_span_content(body: &str) -> String {
    sha256_prefixed(
        normalize_line_endings(body)
            .trim_end_matches('\n')
            .as_bytes(),
    )
}

pub fn checksum_managed_body_line_refs(body_lines: &[&str]) -> String {
    let body_lines = trim_trailing_empty_lines(body_lines);
    let mut hasher = Sha256::new();
    for (index, line) in body_lines.iter().enumerate() {
        if index > 0 {
            hasher.update(b"\n");
        }
        update_line_endings_normalized(&mut hasher, line);
    }
    prefixed_hex_digest(hasher.finalize())
}

fn trim_trailing_empty_lines<'a>(mut lines: &'a [&'a str]) -> &'a [&'a str] {
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines = &lines[..lines.len() - 1];
    }
    lines
}

fn update_line_endings_normalized(hasher: &mut Sha256, line: &str) {
    if !line.contains('\r') {
        hasher.update(line.as_bytes());
        return;
    }
    hasher.update(normalize_line_endings(line).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{checksum_managed_body_line_refs, checksum_managed_span_content};

    #[test]
    fn checksum_trims_trailing_newlines() {
        let checksum = checksum_managed_span_content("alpha\nbeta\n\n");
        assert_eq!(checksum, checksum_managed_span_content("alpha\nbeta"));
    }

    #[test]
    fn checksum_body_line_refs_matches_joined_content() {
        let joined = checksum_managed_span_content("alpha\nbeta\n\n");
        let refs = checksum_managed_body_line_refs(&["alpha", "beta", "", ""]);
        assert_eq!(joined, refs);

        let blank_line = checksum_managed_span_content("alpha\n\nbeta");
        assert_eq!(
            blank_line,
            checksum_managed_body_line_refs(&["alpha", "", "beta"])
        );
    }
}
