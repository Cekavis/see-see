# Bug Assessment: 开机启动会显示主窗口

- **Slug**: silent-autostart
- **Created**: 2026-07-31
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 开机启动应该是静默的

## Symptom

启用开机启动后，系统登录时 See See 会显示主窗口。预期是应用只在后台和托盘启动，用户手动启动、托盘打开或再次启动应用时仍显示主窗口。

## Reproduction

1. 在设置中启用“开机启动”。
2. 注销并重新登录系统。
3. See See 自动运行并显示主窗口，而不是仅驻留托盘。

## Suspected Code Paths

- `src-tauri/tauri.conf.json` — 主窗口默认可见，进程启动时会立即创建并显示。
- `src-tauri/src/lib.rs` — 自启动插件未附加启动标记，应用初始化也未根据启动来源决定是否显示主窗口。
- `src-tauri/src/autostart.rs` — macOS 使用 `SMAppService.mainAppService`，需要读取系统登录项 Apple Event 才能识别登录启动。

## Root Cause Hypothesis

**高置信度**：所有启动路径共享同一个默认可见主窗口，且没有识别自启动来源。Windows/Linux 自启动项可通过插件参数标记；macOS 原生登录项会在 `kAEOpenApplication` 事件的 `keyAEPropData` 中提供 `keyAELaunchedAsLogInItem`。

## Proposed Remediation

**Preferred**: 将主窗口改为默认隐藏，仅在普通启动时由 setup 显示。Windows/Linux 自启动项附加 `--autostart`；macOS 读取登录项启动 Apple Event。第二实例回调忽略自启动标记，普通第二次启动继续显示主窗口。

**Files likely to change**:

- `src-tauri/src/lib.rs`
- `src-tauri/src/autostart.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.lock`

**Tests to add or update**:

- 覆盖普通启动显示、参数标记自启动静默、macOS 登录项自启动静默。

## Risks & Considerations

- 主窗口默认隐藏后，普通启动必须显式显示，否则应用会表现为“没有启动”。
- macOS 登录项信号只在启动 Apple Event 处理期间可读取，应在 Tauri setup 中立即判断。
- 注销/登录实测会中断当前会话，自动化环境只能验证判断逻辑和构建。

## Open Questions

- 无。
