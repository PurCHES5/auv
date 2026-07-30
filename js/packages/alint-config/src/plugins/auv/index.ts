import { definePlugin } from "@alint-js/plugin";

import { noModNamesChecksInTestsRule } from "./rules/no-mod-names-checks-in-tests";
import { vacantControlBoundaryRule } from "./rules/no-vacant-control-boundary";
import { noSourceFilesCompareInTestsRule } from "./rules/no-source-files-compare-in-tests";
import { platformCodedTestPathsRule } from "./rules/no-platform-coded-test-paths";
import { nonRuntimeUnitTestsRule } from "./rules/restrict-non-runtime-unit-tests";
import { caseScopedAppGameTestsRule } from "./rules/require-case-scoped-app-game-tests";
import { sideBySideUnitTestsRule } from "./rules/require-side-by-side-unit-tests";
import { unearnedFunctionBoundaryRule } from "./rules/no-unearned-function-boundary";
import { privateSchemaToolkitRule } from "./rules/no-private-schema-toolkit";
import { establishedFoundationRule } from "./rules/prefer-established-foundation";
import { platformScopedAppIntegrationRule } from "./rules/require-platform-scoped-app-integration";
import { unsafeSafetyCommentRule } from "./rules/unsafe-safety-comment";

export { noModNamesChecksInTestsRule } from "./rules/no-mod-names-checks-in-tests";
export { vacantControlBoundaryRule } from "./rules/no-vacant-control-boundary";
export { noSourceFilesCompareInTestsRule } from "./rules/no-source-files-compare-in-tests";
export { platformCodedTestPathsRule } from "./rules/no-platform-coded-test-paths";
export { nonRuntimeUnitTestsRule } from "./rules/restrict-non-runtime-unit-tests";
export { caseScopedAppGameTestsRule } from "./rules/require-case-scoped-app-game-tests";
export { sideBySideUnitTestsRule } from "./rules/require-side-by-side-unit-tests";
export { unearnedFunctionBoundaryRule } from "./rules/no-unearned-function-boundary";
export { privateSchemaToolkitRule } from "./rules/no-private-schema-toolkit";
export { establishedFoundationRule } from "./rules/prefer-established-foundation";
export { platformScopedAppIntegrationRule } from "./rules/require-platform-scoped-app-integration";
export { unsafeSafetyCommentRule } from "./rules/unsafe-safety-comment";

export default definePlugin({
  rules: {
    "no-mod-names-checks-in-tests": noModNamesChecksInTestsRule,
    "no-vacant-control-boundary": vacantControlBoundaryRule,
    "no-source-files-compare-in-tests": noSourceFilesCompareInTestsRule,
    "no-platform-coded-test-paths": platformCodedTestPathsRule,
    "restrict-non-runtime-unit-tests": nonRuntimeUnitTestsRule,
    "require-case-scoped-app-game-tests": caseScopedAppGameTestsRule,
    "require-side-by-side-unit-tests": sideBySideUnitTestsRule,
    "no-unearned-function-boundary": unearnedFunctionBoundaryRule,
    "no-private-schema-toolkit": privateSchemaToolkitRule,
    "prefer-established-foundation": establishedFoundationRule,
    "require-platform-scoped-app-integration": platformScopedAppIntegrationRule,
    "unsafe-safety-comment": unsafeSafetyCommentRule,
  },
});
