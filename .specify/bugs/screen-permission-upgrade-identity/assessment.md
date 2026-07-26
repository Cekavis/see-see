# Bug Assessment: macOS 升级后屏幕录制授权失效

- **Slug**: screen-permission-upgrade-identity
- **Created**: 2026-07-26
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 现在安装新版本之后似乎需要重新授权屏幕权限，但设置里面 See See 是授权了的，需要删掉再手动添加后 app 内才能检测通过，这是什么原因

安装新版 See See 后，macOS“隐私与安全性 > 屏幕与系统音频录制”仍显示 See See 已获授权，但应用内检查未通过。只有删除系统设置里的旧记录并重新添加当前应用后，检查才恢复正常。

## Symptom

macOS 设置按应用名称展示旧的已授权记录，但新安装的 See See 进程不再匹配该记录的代码签名身份，因此 `CGPreflightScreenCaptureAccess()` 对当前进程返回未授权。删除并重新添加会让 TCC 为当前二进制身份创建新记录，所以应用随后可以检测通过。

## Reproduction

1. 为一个本地构建的 See See 版本授予 macOS 屏幕录制权限。
2. 重新构建新版本并安装到 `/Applications/See See.app`。
3. 保留系统设置中原有的 See See 授权开关并启动新版。
4. 应用内权限检查显示未授权。
5. 从系统设置删除 See See，再手动添加当前 `/Applications/See See.app`；应用内检查恢复通过。

## Suspected Code Paths

- `src-tauri/src/capture.rs:29` — macOS 权限状态通过 `CGPreflightScreenCaptureAccess()` 查询；该 API 返回的是当前进程身份是否获 TCC 授权，而不是仅按显示名称或 bundle identifier 查询。
- `src-tauri/tauri.conf.json:5` — bundle identifier 固定为 `app.seesee.desktop`，但固定 identifier 本身不能替代稳定的代码签名身份。
- `src-tauri/tauri.conf.json:27` — bundle 配置没有建立仓库可验证的 Developer ID 签名、公证发布链路。
- `README.md:76` — 文档明确说明 macOS 签名和公证仍需另行完成，说明当前本地构建不能保证跨版本签名身份稳定。
- `.specify/bugs/screen-recording-permission-loop/test.md` — 既有验证已记录 ad-hoc 签名的新构建会获得新的 TCC 身份并可能要求重新授权。

## Root Cause Hypothesis

**高置信度**。当前安装的 0.3.1 应用是 ad-hoc/linker-signed：`codesign` 显示 `Signature=adhoc`、`TeamIdentifier=not set`，其 designated requirement 退化为当前二进制的 CDHash（`cdhash H"0a06204…"`）。CDHash 会随二进制内容变化，因此每次重新构建后，macOS TCC 会把新版视为不同的代码身份，即使应用名称、路径和 `CFBundleIdentifier` 都没有变化。系统设置仍可显示名为 See See 的旧授权记录，但该记录对应旧 CDHash；当前进程的 CoreGraphics preflight 因身份不匹配而正确返回未授权。删除并重新添加实际上是把 TCC 记录更新为新版 CDHash，并非应用检测错误。

## Proposed Remediation

**Preferred**: 建立稳定的 macOS 发布签名链路。所有交付给用户的版本使用同一个 Apple Developer Team 下的 `Developer ID Application` 证书完整签名 `.app`，保持 `app.seesee.desktop` 不变，并完成 hardened runtime、公证和 stapling。发布验证应检查签名不是 ad-hoc、存在预期 TeamIdentifier、designated requirement 基于证书和 bundle identifier 而非单次构建 CDHash，并执行旧版授权后覆盖安装新版的权限保留测试。

本地开发的未签名/ad-hoc 构建无法可靠复用旧版屏幕录制授权；在尚未建立正式签名发布前，应把“删除旧条目并重新添加当前应用”视为临时恢复方式，并明确区分开发构建与正式发布包。

**Alternatives**:

- 使用稳定的 Apple Development 证书签署内部测试包，可改善同一开发团队内测试构建的身份稳定性，但不适合作为面向用户的分发方案。
- 继续 ad-hoc 签名并在每次升级后引导重新授权，实施成本低，但会持续破坏升级体验，且不能解决签名、公证和 Gatekeeper 的发布要求。

**Files likely to change**:

- macOS 构建/发布工作流配置（当前仓库未发现对应文件）
- `src-tauri/tauri.conf.json`（如需声明 hardened runtime、entitlements 或发布配置）
- `README.md` 或发布文档

**Tests to add or update**:

- 构建产物检查：拒绝 `Signature=adhoc`、缺少 TeamIdentifier 或以 CDHash 作为 designated requirement 的正式 macOS 包。
- 升级人工回归：旧版本授权后覆盖安装同证书签名的新版本，确认系统设置无需删除重加且 `CGPreflightScreenCaptureAccess()` 返回 true。
- 签名/公证检查：`codesign --verify --deep --strict`、`spctl` 和 stapled notarization ticket 均通过。

## Risks & Considerations

- 采用正式签名后的第一个版本相对现有 ad-hoc 版本仍会发生一次身份迁移，用户可能仍需最后重新授权一次；之后同一证书和 identifier 的升级才应稳定继承。
- 更换 Developer Team、证书体系、bundle identifier，或在签名要求中引入不兼容变化，仍可能使 TCC 重新识别应用。
- 不应通过修改应用内检测来把未授权伪装成已授权；实际截图仍会被 macOS 阻止。
- 屏幕录制权限属于系统安全状态，自动化测试不能代替至少一次真实 macOS 覆盖升级验证。

## Open Questions

- [NEEDS CLARIFICATION: 用户所说的“新版本”是否都是当前仓库在同一台机器上本地构建的 ad-hoc 包，还是来自某个尚未纳入仓库的签名发布流程？]
