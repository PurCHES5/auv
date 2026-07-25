# Retired NetEase app-local invoke and run-artifact reuse

**Retired:** 2026-07-25

The following experimental NetEase surfaces were removed together:

- `netease.playlist.selectProof`, a fixture-only invoke command that loaded a
  prebuilt `PlaylistSelectResult` instead of executing playlist selection;
- the otherwise empty app-local invoke registry and `auv-netease-music invoke`
  frontend;
- `src/run_artifacts.rs`, which added NetEase-specific publication and read
  wrappers around the shared `auv-tracing` artifact contract;
- the `playlist select/play --scan-uri` input and candidate-id playback path,
  which read a prior run artifact back into an application command;
- `scan_uri` in playlist-list output and the tests that existed only to prove
  those retired seams.

The retained contract is narrower. Application operations execute from live
typed inputs and return app-owned results. When a tracing context has artifact
authority, NetEase evidence and structured results are emitted through
`src/telemetry.rs` into `auv-tracing`. `--store-root` remains the way the CLI
selects a tracing authority; it is not an app-local artifact reader.

Candidate identifiers remain observation facts in scan output, but the NetEase
CLI does not accept them as cross-run action inputs. A future inspector reads
run metadata and artifacts through the shared tracing/inspection model rather
than an application-specific artifact CLI.

Reintroducing app-local invoke requires an owner-approved real operation wired
to the shared runtime execution model. Reintroducing artifact-backed command
inputs requires an owner-approved shared runtime consumer contract; do not
restore the deleted NetEase-specific RunStore reader or fixture proof command.
