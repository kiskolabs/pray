use crate::hashing::normalize_line_endings;
use crate::package_spec::parse_package_spec;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const DERIVED_EMBEDDING_MODEL: &str = "pray-hash-bucket-v1";
const EMBEDDING_DIMENSIONS: usize = 16;
const MAX_SUMMARY_PARTS: usize = 2;
const MAX_TOKEN_COUNT: usize = 512;
const MAX_SNIPPET_LENGTH: usize = 120;
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "have", "in", "is",
    "it", "its", "of", "on", "or", "package", "the", "this", "to", "was", "we", "with", "you",
    "your",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryDerivedMetadata {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possible_effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possible_side_effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embeddings: Vec<RegistryDerivedEmbedding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_count: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryDerivedEmbedding {
    pub model: String,
    pub vector: Vec<i32>,
}

pub fn derive_registry_derived_metadata_from_root(
    root: &Path,
) -> PrayResult<RegistryDerivedMetadata> {
    let spec_path = find_prayspec_file(root)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&normalize_line_endings(&spec_text))?.canonicalized();
    derive_registry_derived_metadata_from_spec(root, &spec)
}

pub fn derive_registry_derived_metadata_from_archive_bytes(
    archive_bytes: &[u8],
) -> PrayResult<RegistryDerivedMetadata> {
    let temp_dir = unique_temp_dir("pray-derived-metadata");
    fs::create_dir_all(&temp_dir)?;
    let unpack_result = unpack_archive_bytes(archive_bytes, &temp_dir)
        .and_then(|_| derive_registry_derived_metadata_from_root(&temp_dir));
    let _ = fs::remove_dir_all(&temp_dir);
    unpack_result
}

fn derive_registry_derived_metadata_from_spec(
    root: &Path,
    spec: &crate::package_spec::PackageSpec,
) -> PrayResult<RegistryDerivedMetadata> {
    let mut summary_candidates = Vec::new();
    if let Some(summary) = spec.summary.as_deref() {
        push_candidate(&mut summary_candidates, summary);
    }
    if let Some(description) = spec.description.as_deref() {
        push_candidate(&mut summary_candidates, description);
    }
    for export in spec.exports.values() {
        if let Some(summary) = export.summary.as_deref() {
            push_candidate(&mut summary_candidates, summary);
        }
    }

    let mut analyzed_text = String::new();
    let mut sample_lines = Vec::new();
    let mut file_count = 0usize;
    let mut character_count = 0usize;
    for file in &spec.files {
        let path = root.join(file);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let normalized = normalize_line_endings(&text);
        if let Some(line) = first_meaningful_line(&normalized) {
            push_candidate(&mut sample_lines, &line);
        }
        file_count += 1;
        character_count += normalized.chars().count();
        analyzed_text.push_str(&normalized);
        analyzed_text.push('\n');
    }

    for line in &sample_lines {
        push_candidate(&mut summary_candidates, line);
    }
    if summary_candidates.is_empty() {
        for token in top_topics(&analyzed_text) {
            push_candidate(&mut summary_candidates, &token);
        }
    }

    let summary = build_summary(&summary_candidates);
    let tokens = tokenize(&analyzed_text);
    let topics = top_topics(&analyzed_text);
    let categories = infer_categories(&summary_candidates.join(" \n"), &topics);
    let possible_effects = infer_possible_effects(&categories);
    let possible_side_effects = infer_possible_side_effects(&analyzed_text, &topics);
    let embeddings = vec![RegistryDerivedEmbedding {
        model: DERIVED_EMBEDDING_MODEL.to_string(),
        vector: hashed_embedding(&tokens),
    }];

    Ok(RegistryDerivedMetadata {
        summary,
        topics,
        categories,
        possible_effects,
        possible_side_effects,
        embeddings,
        file_count: Some(file_count),
        character_count: Some(character_count),
        token_count: Some(tokens.len()),
    })
}

fn unpack_archive_bytes(archive_bytes: &[u8], root: &Path) -> PrayResult<()> {
    let decoder = zstd::stream::read::Decoder::new(Cursor::new(archive_bytes))?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(root)?;
    Ok(())
}

fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
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

fn build_summary(candidates: &[String]) -> String {
    let mut summary = Vec::new();
    let mut seen = BTreeSet::new();
    for candidate in candidates {
        let normalized = candidate.trim();
        if normalized.is_empty() {
            continue;
        }
        let key = normalized.to_lowercase();
        if seen.insert(key) {
            summary.push(truncate(normalized, MAX_SNIPPET_LENGTH));
        }
        if summary.len() == MAX_SUMMARY_PARTS {
            break;
        }
    }
    if summary.is_empty() {
        return "Package metadata".to_string();
    }
    summary.join(" — ")
}

fn top_topics(text: &str) -> Vec<String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for token in tokenize(text) {
        *counts.entry(token).or_insert(0) += 1;
    }
    let mut ranked: Vec<(String, usize)> = counts.into_iter().collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    ranked.into_iter().take(5).map(|(token, _)| token).collect()
}

fn infer_categories(summary_text: &str, topics: &[String]) -> Vec<String> {
    let text = format!("{} {}", summary_text.to_lowercase(), topics.join(" "));
    let mut categories = Vec::new();
    if contains_any(
        &text,
        &["guide", "guidance", "readme", "documentation", "doc"],
    ) {
        categories.push("documentation".to_string());
    }
    if contains_any(&text, &["test", "testing", "fixture", "spec"]) {
        categories.push("testing".to_string());
    }
    if contains_any(&text, &["auth", "security", "ssh", "key", "sign", "trust"]) {
        categories.push("security".to_string());
    }
    if contains_any(
        &text,
        &["automation", "workflow", "render", "pipeline", "agent"],
    ) {
        categories.push("automation".to_string());
    }
    if categories.is_empty() {
        categories.push("general".to_string());
    }
    categories.sort();
    categories.dedup();
    categories
}

fn infer_possible_effects(categories: &[String]) -> Vec<String> {
    let mut effects = Vec::new();
    if categories
        .iter()
        .any(|category| category == "documentation")
    {
        effects.push("clarifies usage".to_string());
    }
    if categories.iter().any(|category| category == "testing") {
        effects.push("improves validation".to_string());
    }
    if categories.iter().any(|category| category == "security") {
        effects.push("surfaces trust-sensitive behavior".to_string());
    }
    if categories.iter().any(|category| category == "automation") {
        effects.push("reduces manual steps".to_string());
    }
    if effects.is_empty() {
        effects.push("improves package discovery".to_string());
    }
    effects
}

fn infer_possible_side_effects(text: &str, topics: &[String]) -> Vec<String> {
    let mut side_effects = Vec::new();
    if contains_any(
        text,
        &[
            "write",
            "overwrite",
            "replace",
            "render",
            "inject",
            "delete",
        ],
    ) {
        side_effects.push("may change generated output or managed files".to_string());
    }
    if contains_any(&topics.join(" "), &["publish", "release", "deploy"]) {
        side_effects.push("may affect downstream package distribution".to_string());
    }
    side_effects.sort();
    side_effects.dedup();
    side_effects
}

fn hashed_embedding(tokens: &[String]) -> Vec<i32> {
    let mut vector = vec![0i32; EMBEDDING_DIMENSIONS];
    for token in tokens.iter().take(MAX_TOKEN_COUNT) {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hasher.finalize();
        let bucket = hash[0] as usize % EMBEDDING_DIMENSIONS;
        vector[bucket] += 1;
    }
    vector
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in text.split(|character: char| !character.is_alphanumeric()) {
        let token = raw.trim().to_lowercase();
        if token.len() < 4 || STOPWORDS.contains(&token.as_str()) {
            continue;
        }
        tokens.push(token);
        if tokens.len() == MAX_TOKEN_COUNT {
            break;
        }
    }
    tokens
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn push_candidate(candidates: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return;
    }
    candidates.push(truncate(trimmed, MAX_SNIPPET_LENGTH));
}

fn first_meaningful_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| truncate(line, MAX_SNIPPET_LENGTH))
}

fn truncate(text: &str, limit: usize) -> String {
    let mut shortened = String::new();
    for character in text.chars().take(limit) {
        shortened.push(character);
    }
    shortened
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}
