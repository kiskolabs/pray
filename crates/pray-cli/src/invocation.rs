use pray_core::project_context::{ProjectInvocationContext, ProjectInvocationOptions};
use pray_core::resolve::{resolve_project_in_context, ResolvedProject};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    static INVOCATION: RefCell<Option<ProjectInvocationContext>> = const { RefCell::new(None) };
}

pub fn initialize(arguments: Vec<String>) -> PrayResult<Vec<String>> {
    let (options, remaining) = parse_global_options(arguments)?;
    let context = ProjectInvocationContext::from_options(options)?;
    INVOCATION.with(|cell| {
        *cell.borrow_mut() = Some(context);
    });
    Ok(remaining)
}

pub fn manifest_path() -> PathBuf {
    invocation_context().manifest_path
}

pub fn lockfile_path() -> PathBuf {
    invocation_context().lockfile_path()
}

pub fn project_root() -> PathBuf {
    invocation_context().project_root
}

pub fn resolve_current_project(options: &ResolveOptions) -> PrayResult<ResolvedProject> {
    let context = invocation_context();
    let mut options = options.clone();
    if options.environment.is_none() {
        options.environment = context.environment.clone();
    }
    resolve_project_in_context(&context.manifest_path, &context.project_root, &options)
}

fn invocation_context() -> ProjectInvocationContext {
    INVOCATION.with(|cell| {
        cell.borrow().clone().unwrap_or_else(|| {
            ProjectInvocationContext::from_current_directory().expect("project invocation context")
        })
    })
}

fn parse_global_options(
    arguments: Vec<String>,
) -> PrayResult<(ProjectInvocationOptions, Vec<String>)> {
    let mut options = ProjectInvocationOptions::default();
    let mut remaining = Vec::new();
    let mut iter = arguments.into_iter();
    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "--path" => {
                options.project_root = Some(require_option_value("--path", iter.next())?);
            }
            "--file-path" => {
                options.manifest_path = Some(require_option_value("--file-path", iter.next())?);
            }
            "--env" | "--environment" => {
                options.environment = Some(require_environment_value(&argument, iter.next())?);
            }
            value if is_top_level_command(value) => {
                remaining.push(argument);
                remaining.extend(iter);
                break;
            }
            other if other.starts_with('-') => {
                remaining.push(argument);
                remaining.extend(iter);
                break;
            }
            _ => {
                remaining.push(argument);
                remaining.extend(iter);
                break;
            }
        }
    }
    Ok((options, remaining))
}

fn require_option_value(flag: &str, value: Option<String>) -> PrayResult<PathBuf> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| PrayError::Usage(format!("{flag} requires a value")))
}

fn require_environment_value(flag: &str, value: Option<String>) -> PrayResult<String> {
    value.ok_or_else(|| PrayError::Usage(format!("{flag} requires a value")))
}

fn is_top_level_command(token: &str) -> bool {
    matches!(
        token,
        "manifest"
            | "init"
            | "prayer"
            | "repo"
            | "install"
            | "add"
            | "remove"
            | "update"
            | "unlock"
            | "render"
            | "plan"
            | "apply"
            | "verify"
            | "drift"
            | "format"
            | "package"
            | "publish"
            | "login"
            | "serve"
            | "confess"
            | "list"
            | "outdated"
            | "explain"
            | "vendor"
            | "clean"
            | "tree"
            | "sync"
            | "trust"
            | "version"
            | "-V"
            | "--version"
            | "-h"
            | "--help"
    )
}
