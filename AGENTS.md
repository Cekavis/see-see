# Repository Guidelines

## Project Structure & Module Organization

See See is a Tauri 2 desktop application. React/TypeScript UI code lives in `src/`: controls are in `components/`, screens in `views/`, and IPC wrappers in `ipc.ts`. Rust code is under `src-tauri/src/`, with adapters in `providers/` and migrations in `src-tauri/migrations/`. Frontend tests sit beside their subjects; Rust tests are in `src-tauri/tests/`, browser tests in `tests/e2e/`, and feature artifacts in `specs/<number>-<name>/`.

## Build, Test, and Development Commands

- `npm install`: install locked dependencies.
- `npm run tauri dev`: run the desktop app with Vite hot reload.
- `npm run build`: type-check and build the frontend.
- `npm test`: run Vitest tests.
- `npm run test:e2e`: run the WebdriverIO Chrome smoke flow.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust tests.
- `npm run lint` / `npm run format:check`: check ESLint and Prettier.
- `npm run tauri build`: produce release bundles.

## Coding Style & Naming Conventions

Follow `.prettierrc.json`: two-space indentation, semicolons, double quotes, and trailing commas. React components and Rust types use `PascalCase`; TypeScript values use `camelCase`; Rust modules and functions use `snake_case`. Keep IPC names aligned between `src/ipc.ts` and `src-tauri/src/commands.rs`. Prefer existing helpers and small direct changes.

UI copy must earn its space. Do not add subtitles, helper text, or descriptions that merely restate a nearby heading, field label, control, or button. Keep supporting copy only when it adds decision-relevant information such as dynamic status, constraints, consequences, privacy or cost implications, recovery guidance, or non-obvious behavior.

## Testing Guidelines

Name frontend tests `*.test.ts[x]` and Rust tests by behavior, for example `history_integration.rs`. Add a focused regression test for behavioral changes. Before delivery, run lint, formatting, tests, and the relevant build. Capture, permission, and packaging work also requires Windows and macOS manual checks.

## Commits, Pull Requests, and Agent Workflow

Use Conventional Commit subjects, for example `fix: handle cancelled capture`. Keep commits atomic. Pull requests must summarize behavior, link the spec or issue, list verification, and include screenshots for UI changes.

Automatically select the matching Spec Kit workflow:

- Features: `speckit-specify` → `speckit-plan` → `speckit-tasks` → `speckit-implement`; add `clarify`, `analyze`, or `converge` when needed.
- Bugs: `speckit-bug-assess` → `speckit-bug-fix` → `speckit-bug-test`. Keep reports in `.specify/bugs/<slug>/`.
- Git: initialize only if needed, detect the remote with `speckit-git-remote`, and use the configured `speckit-git-commit` hooks. Work directly on `master` by default. Run `speckit-git-feature` and `speckit-git-validate` only when the user explicitly requests a separate branch.

Automatically generate names required by these workflows, including bug slugs and feature identifiers. Do not pause to ask the user to confirm a workflow name unless they explicitly request a specific name or an existing artifact would be overwritten.

Every change, including documentation, must be committed atomically and pushed to `origin/master` unless the current request forbids it or explicitly names another branch. Before staging, inspect status and diffs; exclude secrets, environment files, dependencies, logs, and build output.

Behavior changes increment the synchronized SemVer in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` once per session objective: patch for fixes, minor for compatible features, major for incompatible releases. Run relevant checks, build with `npm run tauri build`, and install locally before committing. Documentation-only changes skip versioning, application builds, and installation, but still require a formatting/readback check, commit, and push.

If the objective changes after completion in the same session, reuse the version and prefer a corrective or `git revert` commit over another bump or amending published history. The current request overrides these defaults.

### GitHub Releases

Releases are published by `.github/workflows/release.yml` when an annotated `vX.Y.Z` tag is pushed. The workflow requires repository secrets `APPLE_CERTIFICATE` (the base64-encoded `.p12` for `See See Local Release`), `APPLE_CERTIFICATE_PASSWORD`, `TAURI_SIGNING_PRIVATE_KEY`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; configure them with `gh secret set` and never commit signing material. It validates the tag against all synchronized version files, creates a draft with generated release notes, uploads Windows x64 MSI and NSIS installers plus Apple Silicon and Intel macOS DMGs, generates signed updater artifacts and `latest.json`, verifies all installers and updater platform entries, then publishes the release as latest.

For each release, finish validation and installation, commit and push `master`, then run `git tag -a vX.Y.Z -m "See See vX.Y.Z"` and `git push origin vX.Y.Z`. Never tag an unpushed commit or reuse a published tag. If the workflow fails, keep the draft and rerun only the failed jobs after fixing the cause; use a new patch version if the release was already published.

The updater public key is embedded in `src-tauri/tauri.conf.json`; its encrypted private key must stay outside the repository. On the primary Windows workstation it is stored at `%USERPROFILE%\.tauri\see-see.key`, with its password protected by DPAPI in `see-see.key.password.dpapi`. Signed local release builds must set `TAURI_SIGNING_PRIVATE_KEY` to the private key path or content plus `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Losing or rotating the private key prevents installed versions from accepting future updates, so back it up securely before publishing the first updater-enabled release.

Agents may recommend useful tools or Spec Kit extensions, explaining the benefit and requesting approval before installation.

## Security & Configuration

Never commit API keys, `.env*`, logs, credentials, dependencies, or build output. Store keys only through the application credential store; remote endpoints must use HTTPS.
