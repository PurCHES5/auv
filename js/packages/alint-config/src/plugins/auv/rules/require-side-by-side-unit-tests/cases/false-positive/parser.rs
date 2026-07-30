fn normalize(value: &str) -> String {
  value.trim().to_lowercase()
}

#[cfg(test)]
#[path = "parser_test.rs"]
mod tests;
