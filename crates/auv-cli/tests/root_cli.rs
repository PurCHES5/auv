//! Subprocess coverage for the root AUV CLI boundary.

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_auv")).args(args).output().expect("run root auv binary")
}

fn stdout(output: &std::process::Output) -> String {
  String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
  String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn root_version_exits_zero_and_names_the_package_version() {
  let output = run(&["--version"]);

  assert_eq!(output.status.code(), Some(0), "auv --version must exit 0; stderr={}", stderr(&output));
  assert_eq!(stdout(&output), format!("auv {}\n", env!("CARGO_PKG_VERSION")));
  assert!(stderr(&output).is_empty(), "auv --version must not write stderr: {}", stderr(&output));
}

#[test]
fn root_help_does_not_advertise_supported_app_or_game_frontends() {
  let output = run(&["--help"]);

  assert_eq!(output.status.code(), Some(0), "auv --help must exit 0; stderr={}", stderr(&output));
  let help = stdout(&output);
  for removed_surface in [
    "auv-godot",
    "auv-osu",
    "auv-minecraft",
    "app.textedit.document.write",
  ] {
    assert!(!help.contains(removed_surface), "root help must not advertise {removed_surface}:\n{help}");
  }
}
