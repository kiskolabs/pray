use crate::dotenv::load_dotenv_variables;
use crate::{PrayError, PrayResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const ENV_PROJECT_PATH: &str = "PRAY_PATH";
const ENV_MANIFEST_PATH: &str = "PRAY_FILE_PATH";
const ENV_ENVIRONMENT: &str = "PRAY_ENV";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInvocationContext {
    pub project_root: PathBuf,
    pub manifest_path: PathBuf,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectInvocationOptions {
    pub project_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub environment: Option<String>,
}

impl ProjectInvocationContext {
    pub fn from_current_directory() -> PrayResult<Self> {
        Self::from_options(ProjectInvocationOptions::default())
    }

    pub fn from_options(options: ProjectInvocationOptions) -> PrayResult<Self> {
        let cwd = env::current_dir().map_err(PrayError::from)?;
        let dotenv = load_dotenv_variables(&cwd);
        let project_root_hint = options
            .project_root
            .clone()
            .or_else(|| env_value(ENV_PROJECT_PATH).map(PathBuf::from))
            .or_else(|| dotenv.get(ENV_PROJECT_PATH).cloned().map(PathBuf::from))
            .unwrap_or_else(|| cwd.clone());
        let project_root = canonicalize_path(&cwd, &project_root_hint)?;
        let manifest_hint = options
            .manifest_path
            .clone()
            .or_else(|| env_value(ENV_MANIFEST_PATH).map(PathBuf::from))
            .or_else(|| dotenv.get(ENV_MANIFEST_PATH).cloned().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("Prayfile"));
        let manifest_path = if manifest_hint.is_absolute() {
            manifest_hint
        } else {
            project_root.join(manifest_hint)
        };
        let environment = options
            .environment
            .or_else(|| env_value(ENV_ENVIRONMENT))
            .or_else(|| dotenv.get(ENV_ENVIRONMENT).cloned())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Ok(Self {
            project_root,
            manifest_path,
            environment,
        })
    }

    pub fn lockfile_path(&self) -> PathBuf {
        self.project_root.join("Prayfile.lock")
    }
}

fn env_value(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn canonicalize_path(base: &Path, path: &Path) -> PrayResult<PathBuf> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    fs::canonicalize(&resolved).or(Ok(resolved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn cli_options_override_process_environment() {
        let _guard = env_lock().lock().expect("env lock");
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let temp = env::temp_dir().join(format!("pray-context-test-{stamp}"));
        let project_root = temp.join("project");
        fs::create_dir_all(&project_root).expect("project dir");
        fs::write(project_root.join("Prayfile"), "prayfile \"1\"\n").expect("prayfile");
        env::set_current_dir(&temp).expect("chdir");
        env::set_var(ENV_PROJECT_PATH, "ignored");
        env::set_var(ENV_ENVIRONMENT, "ignored");

        let context = ProjectInvocationContext::from_options(ProjectInvocationOptions {
            project_root: Some(project_root.clone()),
            manifest_path: None,
            environment: Some("development".to_string()),
        })
        .expect("context");

        let expected_root = fs::canonicalize(&project_root).expect("canonical project root");
        assert_eq!(context.project_root, expected_root);
        assert_eq!(
            fs::canonicalize(&context.manifest_path).expect("canonical manifest path"),
            fs::canonicalize(project_root.join("Prayfile")).expect("canonical expected manifest")
        );
        assert_eq!(context.environment.as_deref(), Some("development"));

        env::remove_var(ENV_PROJECT_PATH);
        env::remove_var(ENV_ENVIRONMENT);
        let _ = env::set_current_dir(env::temp_dir());
        let _ = fs::remove_dir_all(&temp);
    }
}
