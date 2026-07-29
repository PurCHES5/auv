use std::sync::Arc;

use auv_tracing::{Context, EmitBytesOptions, EventPayload, FileTracingStore, RunId, configure, dispatcher};

#[derive(serde::Serialize)]
struct Event {
  ok: bool,
}

#[test]
fn file_store_writes_artifact_bytes_before_the_metadata_record() {
  let directory = tempfile::tempdir().unwrap();
  let store = Arc::new(FileTracingStore::open(directory.path()).unwrap());
  let dispatch = configure().tracing_store(store.clone()).build().unwrap();
  let run_id = RunId::new();
  let body = b"artifact".to_vec();
  let emission = dispatcher::with_default(&dispatch, || {
    Context::root(run_id).in_scope(|| {
      let options = EmitBytesOptions::new().with_purpose("auv.test.output").with_content_type("text/plain").with_file_extension("txt");
      auv_tracing::emit_bytes_artifact(options, body.clone()).unwrap()
    })
  });
  let metadata = futures_executor::block_on(emission).unwrap().unwrap();
  futures_executor::block_on(dispatch.flush()).unwrap();

  let path = directory.path().join("artifacts").join(run_id.to_string()).join(format!("{}.txt", metadata.uri().artifact_id()));
  assert_eq!(store.artifact_path(&metadata), path);
  assert_eq!(std::fs::read(path).unwrap(), body);
  let line = std::fs::read_to_string(directory.path().join("records.jsonl")).unwrap();
  let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
  assert_eq!(value["record"]["type"], "artifact");
  assert_eq!(value["record"]["metadata"]["file_extension"], "txt");
}

#[test]
fn file_store_uses_explicit_extensions_without_inferring_them_from_content_type() {
  let cases = [
    ("image/png", Some("png")),
    ("application/json", Some("json")),
    ("text/plain", Some("txt")),
    ("caller declared", Some("tar.gz")),
    ("image/png", None),
  ];

  for (content_type, expected_extension) in cases {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(FileTracingStore::open(directory.path()).unwrap());
    let dispatch = configure().tracing_store(store).build().unwrap();
    let run_id = RunId::new();
    let emission = dispatcher::with_default(&dispatch, || {
      Context::root(run_id).in_scope(|| {
        let options = EmitBytesOptions::new().with_purpose("auv.test.output").with_content_type(content_type);
        let options = match expected_extension {
          Some(extension) => options.with_file_extension(extension),
          None => options,
        };
        auv_tracing::emit_bytes_artifact(options, b"artifact".to_vec()).unwrap()
      })
    });
    let metadata = futures_executor::block_on(emission).unwrap().unwrap();
    futures_executor::block_on(dispatch.flush()).unwrap();

    let mut file_name = metadata.uri().artifact_id().to_string();
    if let Some(extension) = expected_extension {
      file_name.push('.');
      file_name.push_str(extension);
    }
    let path = directory.path().join("artifacts").join(run_id.to_string()).join(file_name);
    assert_eq!(std::fs::read(path).unwrap(), b"artifact", "content type {content_type}");
    assert_eq!(metadata.file_extension(), expected_extension);
  }
}
impl EventPayload for Event {
  const NAME: &'static str = "auv.test.file";
  const VERSION: u32 = 1;
}

#[test]
fn file_store_appends_stable_json_lines_without_exposing_a_reader() {
  let directory = tempfile::tempdir().unwrap();
  let store = Arc::new(FileTracingStore::open(directory.path()).unwrap());
  let dispatch = configure().tracing_store(store).build().unwrap();
  dispatcher::with_default(&dispatch, || Context::root(RunId::new()).in_scope(|| auv_tracing::emit_event!(Event { ok: true })));
  futures_executor::block_on(dispatch.flush()).unwrap();

  let line = std::fs::read_to_string(directory.path().join("records.jsonl")).unwrap();
  let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
  assert_eq!(value["version"], 1);
  assert_eq!(value["record"]["type"], "event");
  assert_eq!(value["record"]["payload"]["ok"], true);
}
