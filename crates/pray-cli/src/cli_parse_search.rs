use crate::Command;
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn parse_search_command(
    mut arguments: std::vec::IntoIter<String>,
) -> PrayResult<Command> {
    let mut query = None;
    let mut source = None;
    let mut root = None;
    let mut url = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--source" => source = Some(option_value(&mut arguments, "--source")?),
            "--root" => root = Some(PathBuf::from(option_value(&mut arguments, "--root")?)),
            "--url" => url = Some(option_value(&mut arguments, "--url")?),
            other if other.starts_with("--") => {
                return Err(PrayError::Usage(format!("unknown search flag: {other}")))
            }
            other if query.is_none() => query = Some(other.to_string()),
            other => {
                return Err(PrayError::Usage(format!(
                    "unexpected search argument: {other}"
                )))
            }
        }
    }
    let query = query.ok_or_else(|| PrayError::Usage("search requires a query".to_string()))?;
    Ok(Command::Search {
        query,
        source,
        root,
        url,
    })
}

fn option_value(arguments: &mut std::vec::IntoIter<String>, flag: &str) -> PrayResult<String> {
    arguments
        .next()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| PrayError::Usage(format!("{flag} requires a value")))
}
