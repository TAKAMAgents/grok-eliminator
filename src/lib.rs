use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const GROK_ENV_KEYS: [&str; 2] = ["GROK_API_KEY", "XAI_API_KEY"];
const CMUX_BUNDLED_BIN: &str = "/Applications/cmux.app/Contents/Resources/bin";
const CMUX_CLEAN_BIN: &str = ".local/share/cmux-without-grok/bin";
const CMUX_GUARD_START: &str = "# grok-eliminator:begin";
const CMUX_GUARD_END: &str = "# grok-eliminator:end";
const CMUX_GUARD: &str = r#"# grok-eliminator:begin
if [[ -o interactive ]]; then
  autoload -Uz add-zle-hook-widget
  _grok_eliminator_cmux_guard() {
    local cmux_bin='/Applications/cmux.app/Contents/Resources/bin'
    local clean_cmux_bin="$HOME/.local/share/cmux-without-grok/bin"
    local -a path_entries

    [[ -n "${CMUX_SURFACE_ID:-}" || ":$PATH:" == *":$cmux_bin:"* ]] || return
    path_entries=("${(@s/:/)PATH}")
    path_entries=("${path_entries[@]:#$cmux_bin}")
    if [[ -d "$clean_cmux_bin" ]] \
      && (( ${path_entries[(Ie)$clean_cmux_bin]} == 0 )); then
      path_entries=("$clean_cmux_bin" "${path_entries[@]}")
    fi
    export PATH="${(j/:/)path_entries}"
    rehash
    unfunction grok 2>/dev/null
    unset _CMUX_GROK_WRAPPER
  }
  add-zle-hook-widget -Uz line-init _grok_eliminator_cmux_guard
fi
# grok-eliminator:end
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeDirectory(PathBuf);

impl HomeDirectory {
    pub fn current() -> Result<Self, CleanupError> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .map(Self)
            .ok_or(CleanupError::MissingHome)
    }

    pub fn new(path: PathBuf) -> Result<Self, CleanupError> {
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(CleanupError::HomeMustBeAbsolute { path })
        }
    }

    fn join(&self, suffix: &str) -> PathBuf {
        self.0.join(suffix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryStatus {
    Absent,
    Present,
    WouldRemove,
    Removed,
    Configured,
    PreservedByDesign,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportEntry {
    pub target: String,
    pub status: EntryStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub operation: String,
    pub entries: Vec<ReportEntry>,
}

impl Report {
    pub fn render_text(&self) -> String {
        let mut output = format!("operation: {}", self.operation);
        for entry in &self.entries {
            output.push('\n');
            output.push_str(&format!(
                "{:>18}  {}  {}",
                format!("{:?}", entry.status).to_lowercase(),
                entry.target,
                entry.detail
            ));
        }
        output
    }
}

#[derive(Debug, Error)]
pub enum CleanupError {
    #[error("HOME is not set to an absolute path")]
    MissingHome,
    #[error("home path must be absolute: {path}")]
    HomeMustBeAbsolute { path: PathBuf },
    #[error("failed to inspect {path}: {source}")]
    Inspect { path: PathBuf, source: io::Error },
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("invalid package metadata at {path}")]
    InvalidPackageMetadata { path: PathBuf },
    #[error("failed to create cmux mirror on this platform")]
    UnsupportedPlatform,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    name: String,
}

#[derive(Debug, Clone)]
pub struct CleanupEngine {
    home: HomeDirectory,
}

impl CleanupEngine {
    pub fn new(home: HomeDirectory) -> Self {
        Self { home }
    }

    pub fn audit(&self) -> Report {
        let mut entries = Vec::new();
        for path in self.known_artifact_paths() {
            let status = match fs::symlink_metadata(&path) {
                Ok(_) => EntryStatus::Present,
                Err(error) if error.kind() == io::ErrorKind::NotFound => EntryStatus::Absent,
                Err(_) => EntryStatus::Failed,
            };
            entries.push(ReportEntry {
                target: path.display().to_string(),
                status,
                detail: artifact_detail(&path),
            });
        }

        for path in self.shell_config_paths() {
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
                Err(_) => String::new(),
            };
            for key in GROK_ENV_KEYS {
                let present = contains_credential_export(&content, key);
                entries.push(ReportEntry {
                    target: format!("{}:{key}", path.display()),
                    status: if present {
                        EntryStatus::Present
                    } else {
                        EntryStatus::Absent
                    },
                    detail: if present {
                        "credential export detected; value not inspected".to_string()
                    } else {
                        "no credential export".to_string()
                    },
                });
            }
        }

        for key in GROK_ENV_KEYS {
            entries.push(self.audit_platform_env(key));
            entries.push(self.audit_process_env(key));
        }

        if cfg!(target_os = "macos") {
            let cmux_wrapper = PathBuf::from(CMUX_BUNDLED_BIN).join("grok");
            if cmux_wrapper.exists() {
                entries.push(ReportEntry {
                    target: cmux_wrapper.display().to_string(),
                    status: EntryStatus::PreservedByDesign,
                    detail: "signed cmux wrapper is preserved; shell reachability is guarded"
                        .to_string(),
                });
            }
        }

        Report {
            operation: "audit".to_string(),
            entries,
        }
    }

    pub fn plan_removal(&self) -> Report {
        let mut report = self.audit();
        report.operation = "remove (dry-run)".to_string();
        for entry in &mut report.entries {
            if entry.status == EntryStatus::Present {
                entry.status = EntryStatus::WouldRemove;
                entry.detail = "would remove with --apply".to_string();
            }
        }
        report
    }

    pub fn apply_removal(&self) -> Report {
        let mut report = Report {
            operation: "remove (applied)".to_string(),
            entries: Vec::new(),
        };
        let mut changed = false;

        for path in self.known_artifact_paths() {
            if path.ends_with("grok-cli") && !self.package_is_grok(&path) {
                continue;
            }
            if path.ends_with("grok-cli")
                && let Some(entry) = self.try_npm_uninstall(&path)
            {
                if entry.status == EntryStatus::Removed {
                    changed = true;
                }
                report.entries.push(entry);
                continue;
            }
            match remove_existing(&path) {
                Ok(true) => {
                    changed = true;
                    report.entries.push(ReportEntry {
                        target: path.display().to_string(),
                        status: EntryStatus::Removed,
                        detail: "removed local Grok artifact".to_string(),
                    });
                }
                Ok(false) => report.entries.push(ReportEntry {
                    target: path.display().to_string(),
                    status: EntryStatus::Absent,
                    detail: "already absent".to_string(),
                }),
                Err(error) => report.entries.push(ReportEntry {
                    target: path.display().to_string(),
                    status: EntryStatus::Failed,
                    detail: error.to_string(),
                }),
            }
        }

        for path in self.shell_config_paths() {
            match self.sanitize_shell_config(&path) {
                Ok(true) => {
                    changed = true;
                    report.entries.push(ReportEntry {
                        target: path.display().to_string(),
                        status: EntryStatus::Removed,
                        detail: "removed Grok credential exports".to_string(),
                    });
                }
                Ok(false) => {}
                Err(error) => report.entries.push(ReportEntry {
                    target: path.display().to_string(),
                    status: EntryStatus::Failed,
                    detail: error.to_string(),
                }),
            }
        }

        if cfg!(target_os = "macos") && Path::new(CMUX_BUNDLED_BIN).join("grok").exists() {
            match self.ensure_cmux_mirror() {
                Ok(true) => {
                    changed = true;
                    report.entries.push(ReportEntry {
                        target: self.home.join(CMUX_CLEAN_BIN).display().to_string(),
                        status: EntryStatus::Configured,
                        detail: "created cmux helper mirror without Grok".to_string(),
                    });
                }
                Ok(false) => {}
                Err(error) => report.entries.push(ReportEntry {
                    target: self.home.join(CMUX_CLEAN_BIN).display().to_string(),
                    status: EntryStatus::Failed,
                    detail: error.to_string(),
                }),
            }
            match self.ensure_cmux_guard() {
                Ok(true) => {
                    changed = true;
                    report.entries.push(ReportEntry {
                        target: self.home.join(".zshrc").display().to_string(),
                        status: EntryStatus::Configured,
                        detail: "installed cmux Grok reachability guard".to_string(),
                    });
                }
                Ok(false) => report.entries.push(ReportEntry {
                    target: self.home.join(".zshrc").display().to_string(),
                    status: EntryStatus::Absent,
                    detail: "cmux Grok reachability guard already installed".to_string(),
                }),
                Err(error) => report.entries.push(ReportEntry {
                    target: self.home.join(".zshrc").display().to_string(),
                    status: EntryStatus::Failed,
                    detail: error.to_string(),
                }),
            }
        }

        for key in GROK_ENV_KEYS {
            report.entries.push(self.clear_platform_env(key));
        }

        report.entries.push(ReportEntry {
            target: "current shell sessions".to_string(),
            status: if changed {
                EntryStatus::Present
            } else {
                EntryStatus::Absent
            },
            detail: "restart existing shells with `exec zsh`".to_string(),
        });
        report
    }

    fn known_artifact_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.home.join(".grok")];
        if cfg!(windows) {
            paths.extend([
                self.home.join(".local/bin/grok.exe"),
                self.home.join("AppData/Roaming/npm/grok.cmd"),
                self.home.join("AppData/Roaming/npm/grok.ps1"),
                self.home.join("AppData/Local/npm/grok.cmd"),
            ]);
        } else {
            paths.extend([
                self.home.join(".local/bin/grok"),
                PathBuf::from("/opt/homebrew/bin/grok"),
                PathBuf::from("/usr/local/bin/grok"),
            ]);
        }
        for prefix in self.npm_prefixes() {
            paths.push(prefix.join("lib/node_modules/@vibe-kit/grok-cli"));
            paths.push(prefix.join("node_modules/@vibe-kit/grok-cli"));
            if cfg!(windows) {
                paths.push(prefix.join("grok.cmd"));
            } else {
                paths.push(prefix.join("bin/grok"));
            }
        }
        if let Some(root) = self.npm_root_from_command() {
            paths.push(root.join("@vibe-kit/grok-cli"));
        }
        paths.sort();
        paths.dedup();
        paths
    }

    fn npm_prefixes(&self) -> Vec<PathBuf> {
        let mut prefixes = vec![self.home.join(".hermes/node"), self.home.join(".local")];
        if !cfg!(windows) {
            prefixes.extend([PathBuf::from("/opt/homebrew"), PathBuf::from("/usr/local")]);
        }
        if let Some(prefix) = self.npm_prefix_from_command() {
            prefixes.push(prefix);
        }
        prefixes
    }

    fn shell_config_paths(&self) -> Vec<PathBuf> {
        let mut paths = [
            ".zshenv",
            ".zprofile",
            ".zshrc",
            ".bash_profile",
            ".bashrc",
            ".profile",
            ".config/fish/config.fish",
        ]
        .into_iter()
        .map(|path| self.home.join(path))
        .collect::<Vec<_>>();
        if cfg!(windows) {
            paths = vec![
                self.home
                    .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
                self.home
                    .join("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
            ];
        }
        paths
    }

    fn package_is_grok(&self, path: &Path) -> bool {
        let metadata = path.join("package.json");
        let Ok(contents) = fs::read_to_string(metadata) else {
            return false;
        };
        match serde_json::from_str::<PackageMetadata>(&contents) {
            Ok(metadata) => metadata.name == "@vibe-kit/grok-cli",
            Err(_) => false,
        }
    }

    fn npm_prefix_from_command(&self) -> Option<PathBuf> {
        let output = Command::new("npm")
            .args(["config", "get", "prefix"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let prefix = String::from_utf8(output.stdout).ok()?;
        let path = PathBuf::from(prefix.trim());
        path.is_absolute().then_some(path)
    }

    fn npm_root_from_command(&self) -> Option<PathBuf> {
        let output = Command::new("npm")
            .args(["root", "--global"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let root = String::from_utf8(output.stdout).ok()?;
        let path = PathBuf::from(root.trim());
        path.is_absolute().then_some(path)
    }

    fn try_npm_uninstall(&self, package_path: &Path) -> Option<ReportEntry> {
        let active_root = self.npm_root_from_command()?;
        if package_path.parent()? != active_root {
            return None;
        }
        let output = Command::new("npm")
            .args(["uninstall", "--global", "@vibe-kit/grok-cli"])
            .output()
            .ok()?;
        if output.status.success() && !package_path.exists() {
            Some(ReportEntry {
                target: package_path.display().to_string(),
                status: EntryStatus::Removed,
                detail: "removed through the npm package manager".to_string(),
            })
        } else {
            Some(ReportEntry {
                target: package_path.display().to_string(),
                status: EntryStatus::Failed,
                detail: "npm refused to remove the package; no direct deletion attempted"
                    .to_string(),
            })
        }
    }

    fn sanitize_shell_config(&self, path: &Path) -> Result<bool, CleanupError> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(source) => {
                return Err(CleanupError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let (mut sanitized, removed) = strip_credential_exports(&content);
        if cfg!(target_os = "macos")
            && path.ends_with(".zshrc")
            && Path::new(CMUX_BUNDLED_BIN).join("grok").exists()
        {
            let (with_guard, guard_added) = add_cmux_guard(&sanitized);
            sanitized = with_guard;
            if guard_added {
                return atomic_write(path, &sanitized).map(|()| true);
            }
        }
        if removed {
            atomic_write(path, &sanitized).map(|()| true)
        } else {
            Ok(false)
        }
    }

    fn ensure_cmux_guard(&self) -> Result<bool, CleanupError> {
        let path = self.home.join(".zshrc");
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(source) => return Err(CleanupError::Read { path, source }),
        };
        let (updated, added) = add_cmux_guard(&content);
        if added {
            atomic_write(&path, &updated)?;
        }
        Ok(added)
    }

    fn ensure_cmux_mirror(&self) -> Result<bool, CleanupError> {
        let source = Path::new(CMUX_BUNDLED_BIN);
        let clean = self.home.join(CMUX_CLEAN_BIN);
        fs::create_dir_all(&clean).map_err(|source| CleanupError::Write {
            path: clean.clone(),
            source,
        })?;

        #[cfg(not(unix))]
        {
            let _ = source;
            let _ = clean;
            return Err(CleanupError::UnsupportedPlatform);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let entries = fs::read_dir(source).map_err(|error| CleanupError::Inspect {
                path: source.to_path_buf(),
                source: error,
            })?;
            let mut changed = false;
            for entry in entries {
                let entry = entry.map_err(|error| CleanupError::Inspect {
                    path: source.to_path_buf(),
                    source: error,
                })?;
                let name = entry.file_name();
                if name == "grok"
                    || !entry
                        .file_type()
                        .map_err(|source| CleanupError::Inspect {
                            path: entry.path(),
                            source,
                        })?
                        .is_file()
                {
                    continue;
                }
                let target = clean.join(&name);
                let source_path = entry.path();
                let already_linked = fs::read_link(&target)
                    .map(|existing| existing == source_path)
                    .unwrap_or(false);
                if already_linked {
                    continue;
                }
                remove_existing(&target).map_err(|source| CleanupError::Write {
                    path: target.clone(),
                    source,
                })?;
                symlink(&source_path, &target).map_err(|source| CleanupError::Write {
                    path: target,
                    source,
                })?;
                changed = true;
            }
            Ok(changed)
        }
    }

    fn audit_platform_env(&self, key: &str) -> ReportEntry {
        let (target, status, detail) = if cfg!(target_os = "macos") {
            let output = Command::new("launchctl").args(["getenv", key]).output();
            match output {
                Ok(output) if output.stdout.iter().all(u8::is_ascii_whitespace) => (
                    format!("launchd:{key}"),
                    EntryStatus::Absent,
                    "value never displayed".to_string(),
                ),
                Ok(_) => (
                    format!("launchd:{key}"),
                    EntryStatus::Present,
                    "value never displayed".to_string(),
                ),
                Err(error) if error.kind() == io::ErrorKind::NotFound => (
                    format!("launchd:{key}"),
                    EntryStatus::Unavailable,
                    "launchctl is unavailable on this machine".to_string(),
                ),
                Err(_) => (
                    format!("launchd:{key}"),
                    EntryStatus::Failed,
                    "launchctl could not be queried".to_string(),
                ),
            }
        } else if cfg!(windows) {
            let output = Command::new("reg")
                .args(["query", "HKCU\\Environment", "/v", key])
                .output();
            match output {
                Ok(output) if output.status.success() => (
                    format!("windows-user-env:{key}"),
                    EntryStatus::Present,
                    "value never displayed".to_string(),
                ),
                Ok(_) => (
                    format!("windows-user-env:{key}"),
                    EntryStatus::Absent,
                    "user environment value absent".to_string(),
                ),
                Err(error) if error.kind() == io::ErrorKind::NotFound => (
                    format!("windows-user-env:{key}"),
                    EntryStatus::Unavailable,
                    "reg.exe is unavailable on this machine".to_string(),
                ),
                Err(_) => (
                    format!("windows-user-env:{key}"),
                    EntryStatus::Failed,
                    "user environment could not be queried".to_string(),
                ),
            }
        } else {
            (
                format!("persistent-env:{key}"),
                EntryStatus::Unavailable,
                "persistent environment storage is shell-specific on this platform".to_string(),
            )
        };
        ReportEntry {
            target,
            status,
            detail,
        }
    }

    fn audit_process_env(&self, key: &str) -> ReportEntry {
        ReportEntry {
            target: format!("current-process-env:{key}"),
            status: if std::env::var_os(key).is_some() {
                EntryStatus::Present
            } else {
                EntryStatus::Absent
            },
            detail: "parent shell requires a restart; value never displayed".to_string(),
        }
    }

    fn clear_platform_env(&self, key: &str) -> ReportEntry {
        let (target, result) = if cfg!(target_os = "macos") {
            (
                format!("launchd:{key}"),
                Command::new("launchctl").args(["unsetenv", key]).status(),
            )
        } else if cfg!(windows) {
            (
                format!("windows-user-env:{key}"),
                Command::new("reg")
                    .args(["delete", "HKCU\\Environment", "/v", key, "/f"])
                    .status(),
            )
        } else {
            return ReportEntry {
                target: format!("persistent-env:{key}"),
                status: EntryStatus::Unavailable,
                detail: "remove the export from the shell profile and restart the shell"
                    .to_string(),
            };
        };
        ReportEntry {
            target,
            status: match result {
                Ok(status) if status.success() => EntryStatus::Removed,
                Ok(_) => EntryStatus::Absent,
                Err(error) if error.kind() == io::ErrorKind::NotFound => EntryStatus::Unavailable,
                Err(_) => EntryStatus::Failed,
            },
            detail: "cleared without reading the value".to_string(),
        }
    }
}

fn artifact_detail(path: &Path) -> String {
    if path.ends_with(".grok") {
        "Grok user configuration".to_string()
    } else if path.ends_with("grok-cli") {
        "global npm Grok CLI package".to_string()
    } else {
        "known Grok executable path".to_string()
    }
}

pub fn contains_credential_export(content: &str, key: &str) -> bool {
    content.lines().any(|line| is_credential_export(line, key))
}

fn is_credential_export(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    let direct = format!("{key}=");
    let exported = format!("export {key}=");
    let powershell = format!("$env:{key}");
    let fish = format!("set -gx {key} ");
    let setx = format!("setx {key} ");
    trimmed.starts_with(&direct)
        || trimmed.starts_with(&exported)
        || (trimmed.starts_with(&powershell) && trimmed.contains('='))
        || trimmed.starts_with(&fish)
        || trimmed.starts_with(&setx)
}

pub fn strip_credential_exports(content: &str) -> (String, bool) {
    let mut removed = false;
    let mut output = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let body = line.trim_end_matches('\n').trim_end_matches('\r');
        if GROK_ENV_KEYS
            .iter()
            .any(|key| is_credential_export(body, key))
        {
            removed = true;
            continue;
        }
        output.push_str(line);
    }
    (output, removed)
}

fn add_cmux_guard(content: &str) -> (String, bool) {
    if content.contains(CMUX_GUARD_START)
        || content.contains(CMUX_GUARD_END)
        || content.contains("_disable_cmux_grok_wrapper")
        || content.contains("cmux-without-grok")
    {
        return (content.to_string(), false);
    }
    let mut output = content.to_string();
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(CMUX_GUARD);
    (output, true)
}

fn remove_existing(path: &Path) -> Result<bool, io::Error> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(true)
}

fn atomic_write(path: &Path, content: &str) -> Result<(), CleanupError> {
    let parent = path.parent().ok_or_else(|| CleanupError::Write {
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"),
    })?;
    fs::create_dir_all(parent).map_err(|source| CleanupError::Write {
        path: parent.to_path_buf(),
        source,
    })?;
    let temp = parent.join(format!(".grok-eliminator-{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(|source| CleanupError::Write {
            path: temp.clone(),
            source,
        })?;
    file.write_all(content.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| CleanupError::Write {
            path: temp.clone(),
            source,
        })?;
    if let Ok(metadata) = fs::metadata(path) {
        let _ = fs::set_permissions(&temp, metadata.permissions());
    }
    fs::rename(&temp, path).map_err(|source| CleanupError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_only_credential_exports() {
        let input = "export XAI_API_KEY=secret\nkeep=1\nGROK_API_KEY=secret2\n";
        let (output, changed) = strip_credential_exports(input);
        assert!(changed);
        assert_eq!(output, "keep=1\n");
    }

    #[test]
    fn does_not_treat_similar_names_as_credentials() {
        assert!(!contains_credential_export(
            "export XAI_API_KEY_NAME=value",
            "XAI_API_KEY"
        ));
    }

    #[test]
    fn strips_powershell_and_fish_assignments() {
        assert!(contains_credential_export(
            "$env:XAI_API_KEY = 'secret'",
            "XAI_API_KEY"
        ));
        assert!(contains_credential_export(
            "set -gx GROK_API_KEY secret",
            "GROK_API_KEY"
        ));
    }

    #[test]
    fn guard_is_added_once() {
        let (first, added) = add_cmux_guard("export PATH=/usr/bin\n");
        assert!(added);
        let (second, added_again) = add_cmux_guard(&first);
        assert!(!added_again);
        assert_eq!(first, second);
    }
}
