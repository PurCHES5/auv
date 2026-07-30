use super::*;

#[test]
fn no_overlay_is_a_global_flag_without_a_value() {
  let parsed = parse_invoke_args(&["display.capture".to_string(), "--no-overlay".to_string()]).unwrap();
  let InvokeCliParse::Invoke { inputs, .. } = parsed else {
    panic!("expected invoke request");
  };

  assert_eq!(inputs.get("overlay").map(String::as_str), Some("false"));
}
