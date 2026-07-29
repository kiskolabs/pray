use pray_core::cli_suggest::TOP_LEVEL_COMMANDS;
use pray_core::{PrayError, PrayResult};
use std::io::{self, Write};

const GLOBAL_FLAGS: &[&str] = &[
    "--help",
    "-h",
    "--version",
    "-V",
    "--path",
    "--file-path",
    "--env",
    "--no-input",
    "--rm",
    "--trust",
    "--global",
];

pub(crate) fn completion_command(shell: &str) -> PrayResult<()> {
    let script = match shell {
        "bash" => bash_script(),
        "zsh" => zsh_script(),
        "fish" => fish_script(),
        _ => {
            return Err(PrayError::Usage(
                "completion requires bash, zsh, or fish\nSee 'pray --help'.".to_string(),
            ));
        }
    };
    let _ = writeln!(io::stdout(), "{script}");
    Ok(())
}

fn command_words() -> String {
    let mut names: Vec<&str> = TOP_LEVEL_COMMANDS.to_vec();
    names.sort_unstable();
    names.join(" ")
}

fn flag_words() -> String {
    GLOBAL_FLAGS.join(" ")
}

fn bash_script() -> String {
    let commands = command_words();
    let flags = flag_words();
    format!(
        r#"# pray bash completion
_pray() {{
  local cur prev
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  prev="${{COMP_WORDS[COMP_CWORD-1]}}"
  local commands="{commands}"
  local flags="{flags}"
  if [[ "$cur" == -* ]]; then
    COMPREPLY=( $(compgen -W "$flags" -- "$cur") )
    return 0
  fi
  if [[ ${{COMP_CWORD}} -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
    return 0
  fi
  case "$prev" in
    --path|--file-path)
      COMPREPLY=( $(compgen -f -- "$cur") )
      ;;
    completion)
      COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") )
      ;;
    *)
      COMPREPLY=( $(compgen -W "$flags $commands" -- "$cur") )
      ;;
  esac
}}
complete -F _pray pray
"#
    )
}

fn zsh_script() -> String {
    let commands = command_words();
    let flags = flag_words();
    format!(
        r#"#compdef pray
_pray() {{
  local -a commands flags
  commands=( {commands} )
  flags=( {flags} )
  if (( CURRENT == 2 )); then
    _describe -t commands 'pray command' commands
    _describe -t flags 'pray flag' flags
    return
  fi
  case "$words[2]" in
    completion)
      _values 'shell' bash zsh fish
      ;;
    *)
      _files
      _describe -t flags 'pray flag' flags
      ;;
  esac
}}
_pray
"#
    )
}

fn fish_script() -> String {
    let mut lines = vec![
        "# pray fish completion".to_string(),
        "complete -c pray -f".to_string(),
    ];
    for flag in GLOBAL_FLAGS {
        if let Some(name) = flag.strip_prefix("--") {
            lines.push(format!("complete -c pray -l {name} -d 'Global option'"));
        } else if let Some(short) = flag.strip_prefix('-') {
            lines.push(format!("complete -c pray -s {short} -d 'Global option'"));
        }
    }
    for command in command_words().split_whitespace() {
        lines.push(format!(
            "complete -c pray -n '__fish_use_subcommand' -a {command} -d 'pray command'"
        ));
    }
    lines.push(
        "complete -c pray -n '__fish_seen_subcommand_from completion' -a 'bash zsh fish' -d 'shell'"
            .to_string(),
    );
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{bash_script, fish_script, zsh_script};

    #[test]
    fn bash_script_mentions_complete_and_install() {
        let script = bash_script();
        assert!(script.contains("complete -F _pray pray"));
        assert!(script.contains("install"));
        assert!(script.contains("completion"));
    }

    #[test]
    fn zsh_and_fish_scripts_list_shells() {
        assert!(zsh_script().contains("bash zsh fish"));
        assert!(fish_script().contains("bash zsh fish"));
        assert!(fish_script().contains("__fish_use_subcommand"));
    }
}
