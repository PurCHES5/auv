use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn tracing_feature_only_enables_optional_media_tracing_without_enabling_defaults() {
  let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
  let output = Command::new(env!("CARGO"))
    .args([
      "metadata",
      "--format-version=1",
      "--no-deps",
      "--manifest-path",
    ])
    .arg(&manifest_path)
    .output()
    .expect("cargo metadata should run");
  assert!(output.status.success(), "cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr));

  let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("cargo metadata should be JSON");
  let package = metadata["packages"]
    .as_array()
    .expect("metadata packages should be an array")
    .iter()
    .find(|package| package["name"] == "auv-netease-music")
    .expect("NetEase package should be present");
  let features = package["features"].as_object().expect("package features should be an object");
  let tracing = features["tracing"]
    .as_array()
    .expect("tracing feature should be an array")
    .iter()
    .map(|member| member.as_str().expect("feature member should be a string"))
    .collect::<BTreeSet<_>>();

  assert_eq!(tracing, BTreeSet::from(["auv-media-macos/tracing"]));
  assert!(features["default"].as_array().expect("default feature should be an array").is_empty());
}
