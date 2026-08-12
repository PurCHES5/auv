# @auv-js/api-client

Generated Fetch bindings for the daemon-owned AUV REST API.

Run `pnpm generate` in this package to regenerate the Swagger document from
Protobuf and then regenerate this package with Hey API. Files under `src/gen`
are generated and must not be edited by hand.

Dynamic Runner invocation and WebSocket streaming are runtime-described
protocols and are intentionally owned by `auv-js`, not this package.

Create a client with `createClient({ baseUrl, headers })`. The current Swagger
document does not yet encode the listener's bearer policy, so standalone
callers must supply `Authorization: Bearer <credential>` for protected remote
operations. `auv-js` does this through its connection abstraction.
