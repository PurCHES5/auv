# Retired NetEase sidebar proof and ViewMemory experiment

**Retired:** 2026-07-25

Two related experimental surfaces were removed from `auv-netease-music`:

- `netease.playlist.sidebarScanProof`, a fixture-only invoke command with no
  production consumer;
- the default-off `AUV_NETEASE_VIEW_MEMORY=1` playlist reacquisition branch,
  including its app-local memory artifact and sidebar reacquire adapter.

The retained production contract is narrower: `playlist ls` may publish a
typed `auv.netease.playlist_sidebar_scan` run artifact, and playlist select/play
may read that caller-supplied scan URI after authority, run, purpose, content
type, digest, schema, and app validation. Selection then reacquires the live row
through rescan replay.

NetEase-specific tracing events and best-effort screenshot/JSON evidence moved
to `src/telemetry.rs`. The remaining app-local artifact publication/read seam
was subsequently retired; see
[`retired-app-local-invoke-and-run-artifact-reuse.md`](retired-app-local-invoke-and-run-artifact-reuse.md).

Reintroducing ViewMemory requires an owner-approved slice naming a current
production consumer and a runtime/read-side contract. Reintroducing a scan
invoke requires a real app operation, not a fixture-only proof command.
