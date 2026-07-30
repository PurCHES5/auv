fn persist_report(report: &Report) -> Result<(), Error> {
  tracing::record(report);
  artifact_store::write(report)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn persists_and_traces_a_report() {
    assert!(persist_report(&Report::sample()).is_ok());
  }
}
