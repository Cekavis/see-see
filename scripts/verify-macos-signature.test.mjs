import assert from "node:assert/strict";
import test from "node:test";

import { validateSignatureDetails } from "./verify-macos-signature.mjs";

const validSignature = `
Identifier=app.seesee.desktop
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20500 size=123 flags=0x10000(runtime) hashes=3+7 location=embedded
Signature size=2048
Authority=See See Local Release
Info.plist entries=24
TeamIdentifier=not set
Sealed Resources version=2 rules=13 files=9
designated => identifier "app.seesee.desktop" and anchor = H"0123456789ABCDEF"
`;

test("accepts a stable self-signed application identity", () => {
  assert.deepEqual(validateSignatureDetails(validSignature).failures, []);
});

test("rejects the ad-hoc CDHash identity that breaks TCC upgrades", () => {
  const details = `
Identifier=see_see-build-hash
CodeDirectory v=20400 flags=0x20002(adhoc,linker-signed)
Signature=adhoc
Info.plist=not bound
TeamIdentifier=not set
Sealed Resources=none
# designated => cdhash H"ABCDEF"
`;
  const { failures } = validateSignatureDetails(details);

  assert.deepEqual(failures, [
    "应用仍是 ad-hoc 签名",
    "签名证书不是“See See Local Release”",
    "Info.plist 未绑定到代码签名",
    "应用资源未被签名封装",
    "designated requirement 仍绑定单次构建的 CDHash",
    "designated requirement 未绑定 app.seesee.desktop",
  ]);
});

test("rejects an unexpected certificate even when the bundle is sealed", () => {
  const details = validSignature.replace(
    "Authority=See See Local Release",
    "Authority=Temporary Certificate",
  );

  assert.deepEqual(validateSignatureDetails(details).failures, [
    "签名证书不是“See See Local Release”",
  ]);
});
