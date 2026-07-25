# Retired: NetEase Music ACP-2 sidebar scan proof

The fixture-only `netease.playlist.sidebarScanProof` invoke command was retired
on 2026-07-25 because it had no production consumer and represented an archived
vertical proof rather than the active AUV core lane.

Historical context and the retirement boundary are preserved in
[`docs/archive/verticals/netease-music/retired-sidebar-scan-and-view-memory.md`](../../../../archive/verticals/netease-music/retired-sidebar-scan-and-view-memory.md).

The product `playlist ls` scan artifact remains supported through the shared
run store; only the fixture-only invoke command was removed.
