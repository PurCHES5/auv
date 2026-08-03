use super::*;
use auv_api_proto::auv::api::daemon::v1::DeviceRef;

fn device(id: &str, name: &str, local: bool) -> Device {
  Device {
    r#ref: Some(DeviceRef {
      device_id: id.to_string(),
    }),
    name: name.to_string(),
    local,
    ..Device::default()
  }
}

#[test]
fn duplicate_device_name_reports_candidate_ids() {
  let devices = [
    device("device_a", "studio", true),
    device("device_b", "studio", false),
  ];
  let error = select_device(
    &devices,
    &ParentContextOptions {
      device_name: Some("studio".to_string()),
      ..ParentContextOptions::default()
    },
  )
  .expect_err("duplicate name must be ambiguous");
  assert!(error.contains("ambiguous"));
  assert!(error.contains("device_a, device_b"));
}

#[test]
fn device_name_and_id_must_select_the_same_device() {
  let devices = [
    device("device_a", "desktop", true),
    device("device_b", "laptop", false),
  ];
  let error = select_device(
    &devices,
    &ParentContextOptions {
      device_name: Some("desktop".to_string()),
      device_id: Some("device_b".to_string()),
      run_id: None,
    },
  )
  .expect_err("different selectors must fail");
  assert!(error.contains("select different Devices"));
}

#[test]
fn run_only_context_inherits_one_device_but_leaves_multi_device_placement_to_the_scheduler() {
  let one = auv_api_proto::auv::api::daemon::v1::Run {
    devices: vec![DeviceRef {
      device_id: "device_remote".to_string(),
    }],
    ..Default::default()
  };
  assert_eq!(inherited_run_device_id(&one), Some("device_remote"));

  let multiple = auv_api_proto::auv::api::daemon::v1::Run {
    devices: vec![
      DeviceRef {
        device_id: "device_a".to_string(),
      },
      DeviceRef {
        device_id: "device_b".to_string(),
      },
    ],
    ..Default::default()
  };
  assert_eq!(inherited_run_device_id(&multiple), None);
}
