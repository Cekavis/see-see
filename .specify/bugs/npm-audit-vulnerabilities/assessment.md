# Bug Assessment: npm development dependency vulnerabilities

- **Slug**: npm-audit-vulnerabilities
- **Created**: 2026-07-25
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

The user asked: “请你调查 npm audit 的结果，总结一下问题，然后进行修复”.

`npm audit --json` reports 23 vulnerable packages: 22 high, 1 moderate, and 0 critical. All findings are in development dependencies; `npm audit --omit=dev` has no production dependency path to the affected packages.

The report expands three underlying advisory families through the WebdriverIO dependency graph:

- `brace-expansion <=5.0.7` — unbounded expansion can exhaust memory and crash the process (GHSA-mh99-v99m-4gvg, CVSS 7.5).
- `serialize-javascript <=7.0.2` — crafted objects can trigger remote code execution (GHSA-5c6j-r48x-rmvq, CVSS 8.1).
- `serialize-javascript >=5.0.0 <7.0.5` — crafted array-like objects can cause CPU exhaustion (GHSA-qj8w-gfj5-8c6v, CVSS 5.9).

The remaining reported package names are metavulnerabilities inherited through `minimatch`, `glob`, `mocha`, `archiver`, `create-wdio`, and the WebdriverIO packages.

## Symptom

The locked development dependency graph fails `npm audit` with 23 findings even though the direct dependencies are current. The expected state is a clean npm audit without weakening or breaking the browser smoke test.

## Reproduction

1. Install the locked dependencies with Node 24 or newer.
2. Run `npm audit --json`.
3. Observe 22 high and 1 moderate findings, rooted in the WebdriverIO/Mocha development toolchain.

## Suspected Code Paths

- `package.json:33` — the five direct `@wdio/*` development dependencies pull in the vulnerable graph.
- `package-lock.json` — locks vulnerable `brace-expansion`, `serialize-javascript`, WebdriverIO, Mocha, Glob, and Archiver versions.
- `tests/e2e/run.mjs:1` — directly imports the WebdriverIO launcher.
- `tests/e2e/primary-flow.spec.ts:1` — uses WebdriverIO globals and assertions.
- `wdio.conf.ts:1` — configures the WebdriverIO local runner, Mocha framework, and spec reporter.

## Root Cause Hypothesis

Confidence: high. The findings are caused by the WebdriverIO 9.30 test stack, which currently depends on several packages whose supported dependency ranges cannot select patched releases. `npm audit fix --dry-run` can only update the top-level `brace-expansion` from 5.0.7 to 5.0.8 and leaves all 23 findings. `npm audit fix --force --dry-run` proposes an internally inconsistent downgrade to `@wdio/cli` 9.16.2 and `@wdio/mocha-framework` 7.7.3 while retaining all 23 findings. A global override to `brace-expansion` 5 would also replace CommonJS-era major versions with an ESM-only package and risks breaking their consumers.

## Proposed Remediation

**Preferred**: Replace the WebdriverIO/Mocha browser smoke-test harness with `playwright-core`, using its Chromium driver against the locally installed stable Chrome channel. Port the existing single primary-flow test to a direct Node runner with strict assertions, preserving the current layout, navigation, IPC-mock, and call-history checks. Remove the five direct `@wdio/*` packages and the obsolete WebdriverIO config, then regenerate the lockfile.

This eliminates the affected dependency graph instead of forcing incompatible transitive versions or accepting a downgrade that does not clear the audit.

**Alternatives**:

- Wait for coordinated upstream releases from WebdriverIO, Mocha, Archiver, Glob, and their dependants. This preserves the harness but leaves known findings unresolved for an unknown period.
- Add broad npm overrides for `brace-expansion` and `serialize-javascript`. This may make the audit green but crosses module-system and SemVer boundaries and can break the test stack at runtime.

**Files likely to change**:

- `package.json`
- `package-lock.json`
- `tests/e2e/run.mjs`
- `tests/e2e/primary-flow.spec.ts`
- `wdio.conf.ts`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- Port the existing E2E assertions to the Playwright-driven Node runner.
- Run `npm audit`, `npm test`, `npm run lint`, `npm run format:check`, `npm run build`, and `npm run test:e2e`.
- Run the Rust test suite and the required Tauri release build/install verification.

## Risks & Considerations

- The E2E runner will require an installed stable Chrome binary on macOS and Windows, matching the existing test’s Chrome requirement.
- Selector and assertion semantics must remain equivalent during the port.
- The security issue is limited to developer/CI tooling rather than shipped application runtime, so the project impact is medium despite high upstream CVSS scores.
- The existing uncommitted `package-lock.json` diff only removes `libc` metadata from optional binary packages; dependency regeneration may subsume that user-owned formatting change and must be called out before commit.
- This behavior/tooling fix requires a synchronized patch version bump under repository policy.

## Open Questions

- None.
