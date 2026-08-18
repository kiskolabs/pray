use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) struct Claim {
    pub reserved: bool,
    pub slug: String,
}

pub(crate) fn check_rfc_tree(rfc_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let drafts = collect_drafts(rfc_root, &mut errors);
    let claims = collect_claims(&rfc_root.join("ids"), &mut errors);
    for (number, paths) in &drafts {
        if paths.len() > 1 {
            errors.push(format!("RFC {number} is claimed by {}", paths.join(", ")));
        }
        if !claims.contains_key(number) {
            errors.push(format!("RFC {number} has no rfcs/ids/{number} claim"));
        }
    }
    for (number, claim) in &claims {
        let expected = format!("{number}-{}.md", claim.slug);
        let paths = drafts.get(number).cloned().unwrap_or_default();
        if claim.reserved {
            if !paths.is_empty() {
                errors.push(format!(
                    "rfcs/ids/{number} is reserved for {} but {} exists",
                    claim.slug,
                    paths.join(", ")
                ));
            }
            continue;
        }
        match paths.as_slice() {
            [] => errors.push(format!(
                "rfcs/ids/{number} claims {} but {expected} is missing",
                claim.slug
            )),
            [path] if *path != expected => errors.push(format!(
                "rfcs/ids/{number} claims {} but the draft is {path}",
                claim.slug
            )),
            _ => {}
        }
    }
    errors
}

fn collect_drafts(rfc_root: &Path, errors: &mut Vec<String>) -> BTreeMap<String, Vec<String>> {
    let mut drafts = BTreeMap::<String, Vec<String>>::new();
    for entry in fs::read_dir(rfc_root).expect("read rfcs") {
        let name = entry.expect("dirent").file_name();
        let name = name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        let Some((number, slug)) = stem.split_once('-') else {
            continue;
        };
        if number.len() != 4 || !number.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }
        if !slug_is_legal(slug) {
            errors.push(format!("{name} has an illegal slug"));
            continue;
        }
        drafts
            .entry(number.to_string())
            .or_default()
            .push(name.into_owned());
    }
    drafts
}

fn collect_claims(ids_root: &Path, errors: &mut Vec<String>) -> BTreeMap<String, Claim> {
    let mut claims = BTreeMap::new();
    if !ids_root.is_dir() {
        return claims;
    }
    for entry in fs::read_dir(ids_root).expect("read ids") {
        let path = entry.expect("dirent").path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if name.len() != 4 || !name.chars().all(|ch| ch.is_ascii_digit()) {
            errors.push(format!("unexpected file in rfcs/ids: {name}"));
            continue;
        }
        let text = fs::read_to_string(&path).expect("read claim");
        match parse_claim(&text) {
            Ok(claim) => {
                claims.insert(name.to_string(), claim);
            }
            Err(message) => errors.push(format!("rfcs/ids/{name}: {message}")),
        }
    }
    claims
}

fn parse_claim(text: &str) -> Result<Claim, String> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let Some(line) = lines.next() else {
        return Err("claim is empty".to_string());
    };
    if lines.next().is_some() {
        return Err("claim must be one line".to_string());
    }
    if let Some(slug) = line.strip_prefix("reserved ") {
        if slug_is_legal(slug) {
            return Ok(Claim {
                reserved: true,
                slug: slug.to_string(),
            });
        }
        return Err("reserved slug is illegal".to_string());
    }
    if slug_is_legal(line) {
        return Ok(Claim {
            reserved: false,
            slug: line.to_string(),
        });
    }
    Err("slug is illegal".to_string())
}

fn slug_is_legal(slug: &str) -> bool {
    !slug.is_empty()
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        })
}
