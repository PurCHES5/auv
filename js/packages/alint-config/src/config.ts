import { createApeiraAdapter } from "@alint-js/agent-apeira";
import { defineConfig } from "@alint-js/plugin";
import auv from "./plugins/auv";

export default defineConfig([
  {
    name: "auv/rust",
    directories: ["crates/*"],
    files: ["**/*.rs"],
    language: "plaintext",
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/no-vacant-control-boundary": "warn",
      "rust/no-private-schema-toolkit": "warn",
      "rust/no-unearned-function-boundary": "warn",
      "rust/prefer-established-foundation": "warn",
      "rust/unsafe-safety-comment": "warn",
    },
  },
  {
    name: "auv/rust-test-contracts",
    files: ["**/{src,tests,examples}/**/*.rs"],
    language: "plaintext",
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/no-mod-names-checks-in-tests": "error",
      "rust/no-source-files-compare-in-tests": "error",
      "rust/no-platform-coded-test-paths": "error",
    },
  },
  {
    name: "auv/side-by-side-rust-unit-tests",
    files: [
      "**/{src,examples}/**/*.rs",
    ],
    language: "plaintext",
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/require-side-by-side-unit-tests": "error",
    },
  },
  {
    name: "auv/app-game-test-organization",
    files: [
      "supported/**/{src,tests,examples}/**/*.rs",
    ],
    language: "plaintext",
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/require-case-scoped-app-game-tests": "error",
    },
  },
  {
    name: "auv/non-runtime-test-ownership",
    files: [
      "**/{src,tests,examples}/**/*.rs",
    ],
    language: "plaintext",
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/restrict-non-runtime-unit-tests": "error",
    },
  },
  {
    name: "auv/app-integration-directories",
    directories: [
      "supported/**",
    ],
    agent: createApeiraAdapter(),
    plugins: {
      rust: auv,
    },
    rules: {
      "rust/require-platform-scoped-app-integration": 'off',
    },
  },
]);
