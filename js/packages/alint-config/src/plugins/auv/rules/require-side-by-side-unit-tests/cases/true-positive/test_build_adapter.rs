trait Clock {
  fn now(&self) -> u64;
}

#[cfg(test)]
struct FixedClock(u64);

#[cfg(test)]
impl Clock for FixedClock {
  fn now(&self) -> u64 {
    self.0
  }
}
