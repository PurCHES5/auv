#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn maps_the_bottom_right_corner() {
    assert_eq!(project(Point::new(10.0, 20.0), Scale::new(2.0, 0.5)), Point::new(20.0, 10.0));
  }
}
