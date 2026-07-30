use auv_cli::product_registry;

#[test]
fn product_registry_contains_only_core_commands() {
  let registry = product_registry();
  let core = auv_cli_invoke::default_registry();

  assert_eq!(
    registry.all().iter().map(|command| command.id).collect::<Vec<_>>(),
    core.all().iter().map(|command| command.id).collect::<Vec<_>>()
  );
  assert!(registry.all().iter().all(|command| {
    !command.id.starts_with("app.textedit.")
      && !command.id.starts_with("game.minecraft.")
      && !command.id.starts_with("game.osu.")
      && !command.id.starts_with("app.godot.")
  }));
}
