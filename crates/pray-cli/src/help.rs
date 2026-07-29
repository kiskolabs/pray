use crate::help_text::{
    command_help_text, DISTRIBUTION_COMMANDS, GLOBAL_OPTIONS, INSPECT_COMMANDS, META_COMMANDS,
    PACKAGE_COMMANDS, TRUST_COMMANDS, WORKFLOW_COMMANDS,
};
use pray_core::cli_suggest::unknown_command_message;
use pray_core::{PrayError, PrayResult};
use std::io::{self, Write};

pub fn print_concise_help() {
    let _ = writeln!(io::stdout(), "Usage: pray [OPTIONS] <COMMAND>");
    let _ = writeln!(io::stdout());
    let _ = writeln!(
        io::stdout(),
        "Declare shared instructions in Prayfile, lock versions, and render tool-specific output."
    );
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "Getting started:");
    let _ = writeln!(io::stdout(), "  pray init");
    let _ = writeln!(io::stdout(), "  pray install");
    let _ = writeln!(io::stdout(), "  pray plan");
    let _ = writeln!(io::stdout(), "  pray apply");
    let _ = writeln!(io::stdout(), "  pray verify");
    let _ = writeln!(io::stdout());
    print_command_groups();
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "Options:");
    for line in GLOBAL_OPTIONS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout());
    let _ = writeln!(
        io::stdout(),
        "See 'pray help <command>' or 'pray <command> --help' for details on a command."
    );
}

pub fn print_command_help(command: &str) -> bool {
    if let Some(text) = command_help_text(command) {
        let _ = writeln!(io::stdout(), "{text}");
        true
    } else {
        false
    }
}

fn print_command_groups() {
    print_group("Workflow", WORKFLOW_COMMANDS);
    let _ = writeln!(io::stdout());
    print_group("Packages", PACKAGE_COMMANDS);
    let _ = writeln!(io::stdout());
    print_group("Distribution", DISTRIBUTION_COMMANDS);
    let _ = writeln!(io::stdout());
    print_group("Trust", TRUST_COMMANDS);
    let _ = writeln!(io::stdout());
    print_group("Inspect", INSPECT_COMMANDS);
    let _ = writeln!(io::stdout());
    print_group("Meta", META_COMMANDS);
}

fn print_group(title: &str, lines: &[&str]) {
    let _ = writeln!(io::stdout(), "{title}:");
    for line in lines {
        let _ = writeln!(io::stdout(), "  {line}");
    }
}

pub(crate) fn maybe_print_help(arguments: &[String]) -> PrayResult<Option<()>> {
    if arguments.is_empty() {
        print_concise_help();
        return Ok(Some(()));
    }

    if arguments.len() == 1 && matches!(arguments[0].as_str(), "help" | "-h" | "--help") {
        print_concise_help();
        return Ok(Some(()));
    }

    if arguments[0] == "help" {
        let target = arguments.get(1).map(String::as_str).unwrap_or("");
        if matches!(target, "" | "-h" | "--help") {
            print_concise_help();
            return Ok(Some(()));
        }
        if print_command_help(target) {
            return Ok(Some(()));
        }
        return Err(PrayError::Usage(unknown_command_message(target)));
    }

    if let Some(position) = arguments
        .iter()
        .position(|argument| argument == "--help" || argument == "-h")
    {
        if position == 0 {
            print_concise_help();
            return Ok(Some(()));
        }
        let command = &arguments[0];
        if print_command_help(command) {
            return Ok(Some(()));
        }
        return Err(PrayError::Usage(unknown_command_message(command)));
    }

    Ok(None)
}
