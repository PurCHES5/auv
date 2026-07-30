use auv_cli_invoke::{InvokeNamespace, default_registry, render_command_help};

#[test]
fn default_registry_contains_the_scan_commands() {
  let registry = default_registry();

  assert_eq!(registry.resolve("scan.frame").expect("scan.frame").namespace, InvokeNamespace::Scan);
  assert_eq!(registry.resolve("scan.coverage").expect("scan.coverage").namespace, InvokeNamespace::Scan);
}

#[test]
fn every_registered_command_keeps_examples_with_its_typed_help() {
  for command in default_registry().all() {
    let help = render_command_help(command);
    assert!(help.contains("Examples:"), "{} has no inline examples:\n{help}", command.id);
    assert!(help.contains(&format!("auv invoke {}", command.id)), "{} examples do not name the command:\n{help}", command.id);
  }
}
