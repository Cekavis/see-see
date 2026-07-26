import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import process from "node:process";
import { pathToFileURL } from "node:url";

export const DEFAULT_APP_PATH =
  "src-tauri/target/release/bundle/macos/See See.app";
export const EXPECTED_BUNDLE_IDENTIFIER = "app.seesee.desktop";
export const EXPECTED_SIGNING_IDENTITY = "See See Local Release";

export function validateSignatureDetails(
  details,
  {
    expectedBundleIdentifier = EXPECTED_BUNDLE_IDENTIFIER,
    expectedSigningIdentity = EXPECTED_SIGNING_IDENTITY,
  } = {},
) {
  const failures = [];
  const designatedRequirement = details.match(
    /^#?\s*designated => (.+)$/m,
  )?.[1];

  if (
    /Signature=adhoc\b/.test(details) ||
    /\bflags=.*\badhoc\b/.test(details)
  ) {
    failures.push("应用仍是 ad-hoc 签名");
  }
  if (!details.includes(`Authority=${expectedSigningIdentity}`)) {
    failures.push(`签名证书不是“${expectedSigningIdentity}”`);
  }
  if (/Info\.plist=not bound/.test(details)) {
    failures.push("Info.plist 未绑定到代码签名");
  }
  if (/Sealed Resources=none/.test(details)) {
    failures.push("应用资源未被签名封装");
  }
  if (!designatedRequirement) {
    failures.push("缺少 designated requirement");
  } else {
    if (/^cdhash\b/.test(designatedRequirement)) {
      failures.push("designated requirement 仍绑定单次构建的 CDHash");
    }
    if (
      !designatedRequirement.includes(
        `identifier "${expectedBundleIdentifier}"`,
      )
    ) {
      failures.push(
        `designated requirement 未绑定 ${expectedBundleIdentifier}`,
      );
    }
  }

  return { designatedRequirement, failures };
}

export function verifyMacosSignature(
  appPath = DEFAULT_APP_PATH,
  expectedSigningIdentity = EXPECTED_SIGNING_IDENTITY,
) {
  if (process.platform !== "darwin") {
    throw new Error("macOS 签名只能在 macOS 主机上验证");
  }
  if (!existsSync(appPath)) {
    throw new Error(`找不到应用产物：${appPath}`);
  }

  execFileSync(
    "/usr/bin/codesign",
    ["--verify", "--deep", "--strict", appPath],
    { stdio: "pipe" },
  );
  const inspection = spawnSync(
    "/usr/bin/codesign",
    ["-d", "--verbose=4", "--requirements", "-", appPath],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  );
  if (inspection.error) throw inspection.error;
  if (inspection.status !== 0) {
    throw new Error(
      `读取 macOS 签名失败：${inspection.stderr || inspection.stdout}`,
    );
  }
  const details = `${inspection.stdout}${inspection.stderr}`;
  const { designatedRequirement, failures } = validateSignatureDetails(
    details,
    { expectedSigningIdentity },
  );

  if (failures.length > 0) {
    throw new Error(`macOS 签名验证失败：\n- ${failures.join("\n- ")}`);
  }

  return { appPath, designatedRequirement, expectedSigningIdentity };
}

function runCli() {
  const appPath = process.argv[2] ?? DEFAULT_APP_PATH;
  const result = verifyMacosSignature(appPath);
  process.stdout.write(
    [
      `签名验证通过：${result.appPath}`,
      `证书：${result.expectedSigningIdentity}`,
      `Designated requirement：${result.designatedRequirement}`,
      "",
    ].join("\n"),
  );
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  try {
    runCli();
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
