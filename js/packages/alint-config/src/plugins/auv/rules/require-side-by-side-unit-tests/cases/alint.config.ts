import { createApeiraAdapter } from "@alint-js/agent-apeira";
import { defineConfig } from "@alint-js/plugin";

import auv from "../../../index";

export default defineConfig([
  {
    files: ["**/*.rs"],
    language: "text/plain",
    agent: createApeiraAdapter(),
    plugins: { rust: auv },
    rules: { "rust/require-side-by-side-unit-tests": "error" },
  },
]);
