use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn load_dotenv_variables(project_root_hint: &Path) -> BTreeMap<String, String> {
    let path = project_root_hint.join(".env");
    if !path.is_file() {
        return BTreeMap::new();
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return BTreeMap::new(),
    };
    parse_dotenv_text(&text)
}

fn parse_dotenv_text(text: &str) -> BTreeMap<String, String> {
    let mut variables = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let assignment = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        let Some((key, value)) = assignment.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        variables.insert(key.to_string(), parse_dotenv_value(value.trim()));
    }
    variables
}

fn parse_dotenv_value(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let quote = bytes[0];
        if (quote == b'"' || quote == b'\'') && bytes[bytes.len() - 1] == quote {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_dotenv_forms() {
        let variables = parse_dotenv_text(
            r#"
# comment
export PRAY_ENV=development
PRAY_PATH="/tmp/project"
PRAY_FILE_PATH='configs/Prayfile'
"#,
        );
        assert_eq!(
            variables.get("PRAY_ENV").map(String::as_str),
            Some("development")
        );
        assert_eq!(
            variables.get("PRAY_PATH").map(String::as_str),
            Some("/tmp/project")
        );
        assert_eq!(
            variables.get("PRAY_FILE_PATH").map(String::as_str),
            Some("configs/Prayfile")
        );
    }
}
