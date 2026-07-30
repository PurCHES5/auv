#[test]
fn core_registry_excludes_supported_app_and_game_commands() {
  let registry = auv_cli_invoke::default_registry();

  assert!(registry.all().iter().all(|command| {
    !command.id.starts_with("app.textedit.")
      && !command.id.starts_with("game.minecraft.")
      && !command.id.starts_with("game.osu.")
      && !command.id.starts_with("app.godot.")
  }));
}
