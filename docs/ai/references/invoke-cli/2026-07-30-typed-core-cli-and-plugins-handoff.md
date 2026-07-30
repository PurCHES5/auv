# Typed Core CLI and External Plugins Handoff

Issue #149 replaces the prototype root parser and invoke argument shadow schema
with clap-owned command declarations.

## Current boundary

- `auv-cli` owns only the root core frontend: doctor, permissions, invoke,
  lightweight session serving, MCP serving, plugin inspection, and the hidden
  Swift bridge xtask.
- Unknown top-level names resolve to `auv-<name>` executables on `PATH`.
  Built-ins win; only the first name is mapped; arguments and process behavior
  remain native. `auv plugin list` reports precedence, shadowing,
  non-executable candidates, and built-in collisions.
- Supported application and game packages are not linked into `auv-cli`. They
  own independently installed frontends and depend directly on their command,
  driver, and tracing crates. There is no shared `auv-runtime` package.
- `auv-cli-invoke` retains the domain-grouped core registry. Every registered
  command declares a local `clap::Args` input next to its handler. Clap owns CLI
  parsing and long help, primary text operands are positional, and normal
  options use kebab case.
- `#[invoke_command]` binds the local typed input to the handler and registry.
  It has no argument metadata DSL. MCP protocol maps deserialize directly into
  the same typed input rather than simulating CLI argv. CLI and MCP invoke the
  same registered handler and consume its direct-result envelope; only CLI
  renders the optional report, while MCP disables incidental live overlays.
- `auv-cli` also owns the built-in MCP and session frontends. Direct results
  remain owned by typed commands; recording remains owned by `auv-tracing`.

The accepted public evidence seams are root subprocess tests, isolated PATH
plugin fixtures, invoke command parsing/help tests, the downstream macro test,
and existing registry/recording tests.
