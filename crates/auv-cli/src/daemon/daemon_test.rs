use super::*;

#[cfg(unix)]
#[test]
fn first_party_local_runner_receives_daemon_owned_state_root() {
  use auv_api_server::runner_provider::RunnerRuntime;

  let temporary = tempfile::tempdir().expect("temporary daemon store");
  let runtimes = first_party_runner_runtimes(temporary.path()).expect("first-party runtimes");
  let Some(RunnerRuntime::Executable(local)) = runtimes.local_driver else {
    panic!("local driver must use an executable runtime");
  };
  let expected = temporary.path().join("runner-state").join("auv.core.local");
  assert_eq!(local.environment.get(crate::runner::STATE_ROOT_ENV).map(String::as_str), expected.to_str());
}
