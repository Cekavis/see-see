import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import process from "node:process";

const read = (path) => readFileSync(path, "utf8");
const workflow = read(".github/workflows/release.yml");
const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));
const capability = JSON.parse(read("src-tauri/capabilities/default.json"));
const packageJson = JSON.parse(read("package.json"));
const cargo = read("src-tauri/Cargo.toml");

assert.equal(tauri.bundle.createUpdaterArtifacts, true);
assert.deepEqual(tauri.plugins.updater.endpoints, [
  "https://github.com/Cekavis/see-see/releases/latest/download/latest.json",
]);
assert.match(tauri.plugins.updater.pubkey, /^[A-Za-z0-9+/=]+$/);
assert.ok(capability.permissions.includes("updater:default"));
assert.ok(capability.permissions.includes("process:allow-restart"));
assert.ok(packageJson.dependencies["@tauri-apps/plugin-updater"]);
assert.ok(packageJson.dependencies["@tauri-apps/plugin-process"]);
assert.match(cargo, /^tauri-plugin-updater = "2"$/m);
assert.match(cargo, /^tauri-plugin-process = "2"$/m);

for (const required of [
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
  "uploadUpdaterJson: true",
  "updaterJsonPreferNsis: true",
  'gh release download "$GITHUB_REF_NAME" --pattern latest.json',
  '"windows-x86_64"',
  '"darwin-aarch64"',
  '"darwin-x86_64"',
  'gh release edit "$GITHUB_REF_NAME" --draft=false --latest',
]) {
  assert.ok(
    workflow.includes(required),
    `release workflow is missing ${required}`,
  );
}

process.stdout.write("release updater configuration is complete\n");
