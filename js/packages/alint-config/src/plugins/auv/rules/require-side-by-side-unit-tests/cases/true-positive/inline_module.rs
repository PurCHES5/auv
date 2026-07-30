fn normalize(value: &str) -> String {
  value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn trims_input() {
    assert_eq!(normalize(" A "), "a");
  }
}
