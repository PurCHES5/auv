//! Discovery and execution of external `auv-*` command plugins.

use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILTIN_COMMANDS: &[&str] = &["doctor", "invoke", "session", "mcp", "plugin"];

pub fn execute(command_name: &OsStr, arguments: &[OsString]) -> Result<i32, String> {
  let executable_name = executable_name(command_name);
  let executable = resolve(&executable_name)
    .ok_or_else(|| format!("unknown command {:?}; no {:?} executable was found on PATH", command_name, executable_name))?;
  let auv_path = env::current_exe().map_err(|error| format!("failed to resolve the auv executable path: {error}"))?;

  execute_resolved(&executable, arguments, &auv_path)
}

#[cfg(unix)]
fn execute_resolved(executable: &Path, arguments: &[OsString], auv_path: &Path) -> Result<i32, String> {
  use std::os::unix::process::CommandExt;

  let error = Command::new(executable).args(arguments).env("AUV_PATH", auv_path).exec();
  Err(format!("failed to execute plugin {}: {error}", executable.display()))
}

#[cfg(windows)]
fn execute_resolved(executable: &Path, arguments: &[OsString], auv_path: &Path) -> Result<i32, String> {
  let status = Command::new(executable)
    .args(arguments)
    .env("AUV_PATH", auv_path)
    .status()
    .map_err(|error| format!("failed to execute plugin {}: {error}", executable.display()))?;
  Ok(status.code().unwrap_or(1))
}

pub fn list() -> Result<i32, String> {
  let path = env::var_os("PATH").ok_or_else(|| "PATH is not set; no AUV plugins can be discovered".to_string())?;
  let mut seen = HashMap::<OsString, PathBuf>::new();
  let mut warnings = Vec::new();
  let mut plugins = Vec::new();

  for directory in env::split_paths(&path) {
    let Ok(entries) = fs::read_dir(&directory) else {
      continue;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
      let name = entry.file_name();
      let Some(command_name) = plugin_command_name(&name) else {
        continue;
      };
      let command_key = command_name.clone();
      let path = entry.path();
      if !is_executable(&path) {
        warnings.push(format!("{} is named like an AUV plugin but is not executable", path.display()));
        continue;
      }
      if let Some(visible) = seen.get(&command_key) {
        warnings.push(format!("{} is shadowed by {} earlier on PATH", path.display(), visible.display()));
        continue;
      }
      if BUILTIN_COMMANDS.iter().any(|builtin| command_name == OsStr::new(builtin)) {
        warnings.push(format!("{} collides with built-in command `{}`", path.display(), command_name.to_string_lossy()));
      }
      seen.insert(command_key, path.clone());
      plugins.push(path);
    }
  }

  if plugins.is_empty() {
    println!("No AUV plugins were found on PATH.");
  } else {
    println!("The following AUV-compatible plugins are available:");
    for plugin in plugins {
      println!("{}", plugin.display());
    }
  }
  for warning in &warnings {
    eprintln!("warning: {warning}");
  }

  Ok(i32::from(!warnings.is_empty()))
}

fn executable_name(command_name: &OsStr) -> OsString {
  let mut executable = OsString::from("auv-");
  executable.push(command_name);
  executable
}

#[cfg(unix)]
fn resolve(executable_name: &OsStr) -> Option<PathBuf> {
  let path = env::var_os("PATH")?;
  env::split_paths(&path).find_map(|directory| {
    let candidate = directory.join(executable_name);
    is_executable(&candidate).then_some(candidate)
  })
}

#[cfg(windows)]
fn resolve(executable_name: &OsStr) -> Option<PathBuf> {
  let path = env::var_os("PATH")?;
  let extensions = windows_executable_extensions();
  env::split_paths(&path).find_map(|directory| {
    let direct = directory.join(executable_name);
    if is_executable(&direct) {
      return Some(direct);
    }
    extensions.iter().find_map(|extension| {
      let mut candidate_name = executable_name.to_os_string();
      candidate_name.push(extension);
      let candidate = directory.join(candidate_name);
      is_executable(&candidate).then_some(candidate)
    })
  })
}

#[cfg(unix)]
fn plugin_command_name(file_name: &OsStr) -> Option<OsString> {
  use std::os::unix::ffi::{OsStrExt, OsStringExt};

  file_name.as_bytes().strip_prefix(b"auv-").map(|name| OsString::from_vec(name.to_vec()))
}

#[cfg(windows)]
fn plugin_command_name(file_name: &OsStr) -> Option<OsString> {
  let path = Path::new(file_name);
  path.extension()?;
  path.file_stem()?.to_str()?.strip_prefix("auv-").map(OsString::from)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
  use std::os::unix::fs::PermissionsExt;

  fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
  path.is_file()
    && path.extension().and_then(OsStr::to_str).is_some_and(|extension| {
      windows_executable_extensions().iter().any(|candidate| candidate.trim_start_matches('.').eq_ignore_ascii_case(extension))
    })
}

#[cfg(windows)]
fn windows_executable_extensions() -> Vec<String> {
  const SUPPORTED: &[&str] = &[".COM", ".EXE", ".BAT", ".CMD"];
  env::var_os("PATHEXT")
    .map(|value| {
      value
        .to_string_lossy()
        .split(';')
        .filter(|extension| SUPPORTED.iter().any(|supported| supported.eq_ignore_ascii_case(extension)))
        .map(str::to_owned)
        .collect()
    })
    .unwrap_or_else(|| SUPPORTED.iter().map(|extension| (*extension).to_string()).collect())
}
