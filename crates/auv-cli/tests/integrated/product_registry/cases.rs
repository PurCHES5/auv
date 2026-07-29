use auv_cli::product_registry;

const DOCUMENT_WRITE_COMMAND_ID: &str = "app.textedit.document.write";

#[test]
fn product_registry_includes_textedit_once_and_excludes_balatro() {
  let registry = product_registry();

  assert!(registry.resolve(DOCUMENT_WRITE_COMMAND_ID).is_some());
  assert_eq!(registry.all().iter().filter(|command| command.id == DOCUMENT_WRITE_COMMAND_ID).count(), 1);
  assert!(registry.all().iter().all(|command| !command.id.starts_with("game.balatro.")));
}

#[test]
fn core_registry_excludes_textedit() {
  assert!(auv_cli_invoke::default_registry().resolve(DOCUMENT_WRITE_COMMAND_ID).is_none());
}
