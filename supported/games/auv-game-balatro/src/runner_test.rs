use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

#[test]
fn rejects_malformed_rgb_frame_before_model_loading() {
  let error = detect_objects(
    &LazyResourceCache::default(),
    proto::ObjectDetectorSpec {
      detector_id: "test".to_string(),
      model_path: "/missing/model.onnx".to_string(),
      ..Default::default()
    },
    image_proto::RgbFrame {
      width: 2,
      height: 2,
      data: vec![0; 11],
    },
  )
  .expect_err("invalid frame");
  assert_eq!(error.code(), tonic::Code::InvalidArgument);
  assert!(error.message().contains("expected 12"));
}

#[test]
fn lazy_resource_cache_initializes_once_per_key() {
  let cache = LazyResourceCache::<String, usize>::default();
  let loads = AtomicUsize::new(0);
  let first = cache.get_or_try_init("detector".to_string(), |_| Ok(loads.fetch_add(1, Ordering::SeqCst))).expect("first load");
  let second = cache.get_or_try_init("detector".to_string(), |_| Ok(loads.fetch_add(1, Ordering::SeqCst))).expect("cached load");
  assert!(Arc::ptr_eq(&first, &second));
  assert_eq!(loads.load(Ordering::SeqCst), 1);
}
