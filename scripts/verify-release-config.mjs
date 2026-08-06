import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import process from "node:process";

const read = (path) => readFileSync(path, "utf8");
const workflow = read(".github/workflows/release.yml");
const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));
const capability = JSON.parse(read("src-tauri/capabilities/default.json"));
const packageJson = JSON.parse(read("package.json"));
const cargo = read("src-tauri/Cargo.toml");
const tauriActionBlock = workflow
  .split("- uses: tauri-apps/tauri-action@v1")[1]
  ?.split("        with:")[0];

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
  "Import macOS signing certificate",
  'security import "$certificate_path"',
  "security find-key",
  "Remove macOS signing keychain",
  "--bundles app,dmg",
  "\\.app\\.tar\\.gz$",
  "\\.app\\.tar\\.gz\\.sig$",
  'APPLE_SIGNING_IDENTITY: "See See Local Release"',
  "uploadUpdaterJson: true",
  "updaterJsonPreferNsis: true",
  'releaseAssetNamePattern: "See-See_[version]_[platform]_[arch][setup][ext]"',
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

assert.ok(tauriActionBlock, "release workflow is missing the Tauri action");
assert.doesNotMatch(
  tauriActionBlock,
  /APPLE_CERTIFICATE(?:_PASSWORD)?:/,
  "the Tauri action must use the manually imported custom certificate",
);
assert.doesNotMatch(
  workflow,
  /security add-trusted-cert/,
  "the headless release workflow must not mutate certificate trust",
);
assert.doesNotMatch(
  workflow,
  /--notes\b|## 安装包/,
  "release notes must contain only GitHub's generated changelog",
);
assert.doesNotMatch(
  workflow,
  /releaseAssetNamePattern:.*\[name\]|releaseAssetNamePattern:.*\.\[ext\]/,
  "release asset names must not rely on a spaced product name or duplicate the extension separator",
);
assert.equal(
  workflow.match(/--bundles app,dmg/g)?.length,
  2,
  "both macOS architectures must build app updater artifacts and DMGs",
);

process.stdout.write("release updater configuration is complete\n");
