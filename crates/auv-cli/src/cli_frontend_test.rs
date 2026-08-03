use super::*;

#[test]
fn selected_control_run_and_tracing_run_share_one_uuid_identity() {
  let selected = crate::plugin::ResolvedExecutionContext {
    context: auv::AuvContext {
      run_id: Some("019fb9193a0c73d2a06e9f94a150ccfb".to_string()),
      ..Default::default()
    },
    implicit_run_id: None,
  };

  assert_eq!(tracing_run_id_for_selected_context(Some(&selected)).unwrap().to_string(), "019fb919-3a0c-73d2-a06e-9f94a150ccfb");
}

#[test]
fn malformed_selected_control_run_is_not_silently_recorded_as_another_run() {
  let selected = crate::plugin::ResolvedExecutionContext {
    context: auv::AuvContext {
      run_id: Some("external-run".to_string()),
      ..Default::default()
    },
    implicit_run_id: None,
  };

  let error = tracing_run_id_for_selected_context(Some(&selected)).expect_err("malformed selected Run must fail closed");
  assert!(error.contains("cannot be projected"));
}
