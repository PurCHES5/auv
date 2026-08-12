# @auv-js/api-client

Generated Fetch bindings for the daemon-owned AUV REST API.

Run `buf generate` from `proto` to regenerate the Protobuf-ES and Swagger
artifacts together. Then run `pnpm generate` in this package to regenerate the
client from that Swagger document with Hey API. Files under `src/gen` are
generated and must not be edited by hand.

Dynamic Runner invocation and WebSocket streaming are runtime-described
protocols and are intentionally owned by `auv-js`, not this package.

Create a client with `createClient({ baseUrl, headers })`. The current Swagger
document does not yet encode the listener's bearer policy, so standalone
callers must supply `Authorization: Bearer <credential>` for protected remote
operations. `auv-js` does this through its connection abstraction.
