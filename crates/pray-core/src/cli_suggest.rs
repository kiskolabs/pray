pub const TOP_LEVEL_COMMANDS: &[&str] = &[
    "add", "apply", "clean", "confess", "drift", "explain", "format", "help", "init", "install",
    "list", "login", "manifest", "outdated", "package", "plan", "prayer", "publish", "remove",
    "render", "repo", "serve", "sync", "tree", "trust", "unlock", "update", "vendor", "verify",
    "version",
];

pub fn unknown_command_message(command: &str) -> String {
    let mut message = format!("unknown command: {command}");
    if let Some(suggestion) = suggest_command(command, TOP_LEVEL_COMMANDS) {
        message.push_str(&format!("\nDid you mean `{suggestion}`?"));
    }
    message
}

pub fn suggest_command<'a>(input: &str, candidates: &'a [&str]) -> Option<&'a str> {
    let maximum_distance = if input.chars().count() <= 3 { 1 } else { 2 };
    candidates
        .iter()
        .copied()
        .filter(|candidate| levenshtein_distance(input, candidate) <= maximum_distance)
        .min_by_key(|candidate| levenshtein_distance(input, candidate))
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let left_length = left_chars.len();
    let right_length = right_chars.len();

    if left_length == 0 {
        return right_length;
    }
    if right_length == 0 {
        return left_length;
    }

    let mut previous_row: Vec<usize> = (0..=right_length).collect();
    let mut current_row = vec![0; right_length + 1];

    for (left_index, left_character) in left_chars.iter().enumerate() {
        current_row[0] = left_index + 1;
        for (right_index, right_character) in right_chars.iter().enumerate() {
            let substitution_cost = if left_character == right_character {
                0
            } else {
                1
            };
            current_row[right_index + 1] = (previous_row[right_index + 1] + 1)
                .min(current_row[right_index] + 1)
                .min(previous_row[right_index] + substitution_cost);
        }
        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[right_length]
}

#[cfg(test)]
mod tests {
    use super::{suggest_command, unknown_command_message, TOP_LEVEL_COMMANDS};

    #[test]
    fn suggests_install_for_common_typo() {
        assert_eq!(
            suggest_command("instal", TOP_LEVEL_COMMANDS),
            Some("install")
        );
    }

    #[test]
    fn unknown_command_includes_suggestion() {
        let message = unknown_command_message("instal");
        assert!(message.contains("unknown command: instal"));
        assert!(message.contains("Did you mean `install`?"));
    }
}
