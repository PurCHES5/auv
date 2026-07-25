# Retired: NetEase Music ACP-1 playlist select proof

The fixture-only `netease.playlist.selectProof` command, the app-local invoke
registry, and the NetEase-owned run-artifact read/reuse seam were retired on
2026-07-25. They represented an archived vertical proof rather than a current
application operation on AUV's shared runtime model.

The product `playlist ls`, `playlist select`, and `playlist play` operations
remain. With `--store-root`, their evidence is emitted into the active run via
`auv-tracing`; the app CLI no longer returns or accepts a scan artifact URI.

Historical context and the reintroduction boundary are preserved in
[`docs/archive/verticals/netease-music/retired-app-local-invoke-and-run-artifact-reuse.md`](../../../../archive/verticals/netease-music/retired-app-local-invoke-and-run-artifact-reuse.md).
