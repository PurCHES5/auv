use super::*;

#[test]
fn trims_input() {
  assert_eq!(normalize(" A "), "a");
}
