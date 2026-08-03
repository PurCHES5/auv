# Device / Run / Runner API

Device/Run/Runner control API, protobuf, Runner aggregation, MCP frontend, and
the tombstone for the retired SessionService prototype. The folder name is
retained as a stable responsibility path; Session is not a public resource.

Accepted target architecture:
[`2026-08-03-auv-facade-daemon-runner-architecture.md`](2026-08-03-auv-facade-daemon-runner-architecture.md).
The 2026-07-31 aggregated API is the current implementation baseline, not the
accepted package and routing target. Public SessionService, session-scoped
Connection, legacy VisionService, and `/v1/*:verb` routes were removed on
2026-07-31.

Current implementation ownership still places daemon state and typed capability
forwarding in `auv-api-server`, context/placement policy in `auv-api-client`,
and serving composition in `auv-cli`. The accepted target introduces `auv` as
the local/remote domain facade and `auv-daemon` as the long-lived control owner;
`auv-api-client` and `auv-api-server` become protocol boundaries. MCP remains in
`auv-cli`.

Count: **15**

- [`2026-08-03-auv-facade-daemon-runner-architecture.md`](2026-08-03-auv-facade-daemon-runner-architecture.md) — accepted facade, daemon, opaque routing, extension, and Runner target.
- [`2026-07-31-device-run-runner-aggregated-api-design.md`](2026-07-31-device-run-runner-aggregated-api-design.md)
- [`2026-07-31-daemon-session-api-architecture.md`](2026-07-31-daemon-session-api-architecture.md)
- [`2026-08-02-api-client-server-package-architecture-research.md`](2026-08-02-api-client-server-package-architecture-research.md) — primary-source comparison and a capability-scoped client/server alternative.
- [`2026-08-03-rust-server-lifecycle-naming-research.md`](2026-08-03-rust-server-lifecycle-naming-research.md) — primary-source Rust server lifecycle and naming comparison.
- [`2026-08-03-auv-daemon-document-history.md`](2026-08-03-auv-daemon-document-history.md) — Git-backed timeline for when the independent `auv-daemon` owner became the accepted target.

- [`2026-06-10-stateful-session-daemon-js-repl-v0.md`](2026-06-10-stateful-session-daemon-js-repl-v0.md)
- [`2026-06-11-mcp-frontend-surface-v0.md`](2026-06-11-mcp-frontend-surface-v0.md)
- [`2026-06-11-mcp-read-chain-evidence-pack.md`](2026-06-11-mcp-read-chain-evidence-pack.md)
- [`2026-06-18-core-realtime-session-substrate-slice-design.md`](2026-06-18-core-realtime-session-substrate-slice-design.md)
- [`2026-06-18-core-realtime-session-substrate-v0.md`](2026-06-18-core-realtime-session-substrate-v0.md)
- [`2026-06-30-api-session-api-operator-guide.md`](2026-06-30-api-session-api-operator-guide.md)
- [`2026-06-30-api-session-proto-boundary-review.md`](2026-06-30-api-session-proto-boundary-review.md)
- [`2026-06-30-api-session-proto-server-seam-design.md`](2026-06-30-api-session-proto-server-seam-design.md)
- [`2026-06-30-session-api-closeout.md`](2026-06-30-session-api-closeout.md)

## Related

- Parent index: [`../INDEX.md`](../INDEX.md)
- Docs overview: [`../../../README.md`](../../../README.md)
- Shared vocabulary: [`../../../TERMS_AND_CONCEPTS.md`](../../../TERMS_AND_CONCEPTS.md)
