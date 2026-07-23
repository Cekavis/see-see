# Implementation Plan: See See 多模态截图翻译

**Branch**: `master` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-screenshot-ai-translation/spec.md`

## Summary

构建一个本地优先的 Windows/macOS 桌面应用：全局快捷键触发跨显示器区域截图，Rust 后端直接把 PNG 截图和当前提示词发送给用户选择的 OpenAI、Anthropic 或 Gemini 协议端点，并通过 Tauri Channel 把统一的流式事件送到结果浮窗。应用不执行 OCR、不建设账号或中转服务；模型配置、提示词与完整历史保存在本地 SQLite，截图图像单独存入同一数据库的 BLOB 表以获得事务性写入和删除，历史搜索在 10,000 条目标规模内使用参数化文本扫描，API Key 只进入操作系统凭据存储。

## Technical Context

**Language/Version**: Rust 1.95；TypeScript 5.x；Node.js 24 LTS

**Primary Dependencies**: Tauri 2.x、React 19.2、Vite、xcap、image、reqwest、eventsource-stream、rusqlite（bundled SQLite）、keyring；Tauri 官方 global-shortcut、autostart、single-instance、clipboard-manager、opener、dialog、log 插件

**Storage**: 单个本地 SQLite 数据库，保存非秘密设置、模型配置、提示词、历史元数据/文本及截图 BLOB；API Key 存入 Windows Credential Manager 或 macOS Keychain

**Testing**: `cargo test`（领域逻辑、协议序列化、SSE 解析、SQLite 集成）；Vitest + React Testing Library（界面状态和无障碍交互）；WebdriverIO Tauri service（一个关键桌面流程烟雾测试）；Windows/macOS 人工截图与视觉验证

**Target Platform**: Windows 10/11 x64；macOS 14+ universal（Apple Silicon + Intel）

**Project Type**: 单仓库跨平台桌面应用，无远程后端

**Performance Goals**: 快捷键触发后 500 ms 内显示截图选区；框选后 200 ms 内显示提交或错误状态；生成期间界面持续响应；10,000 条历史记录的打开、搜索和筛选在 2 秒内完成

**Constraints**: 单次只允许一个活动请求；不使用 OCR；远程端点必须为 HTTPS，本机回环端点可用 HTTP；不自动重试；不采集遥测；普通配置、历史和日志不得包含 API Key；关闭历史后不得持久化截图或结果

**Scale/Scope**: 单本地用户、3 个协议适配器、2 个内置提示词、约 6 个窗口/视图、10,000 条历史记录验收规模

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-design gate

- **Maintainability — PASS**: 单项目、单数据库、直接协议适配器；不引入后端服务、依赖注入容器、通用插件框架、路由器或全局状态库。每个新增依赖均对应无法由标准库或 Tauri 核心可靠覆盖的系统边界。
- **Testing — PASS**: 计划包含 Rust 单元/集成测试、前端状态测试、一个桌面烟雾测试，以及 Windows/macOS 必需的权限、混合缩放截图和视觉人工验证。
- **User experience — PASS**: 规格已经定义加载、空白、成功、失败、取消、禁用和恢复状态；计划保留统一错误码和单一活动请求状态机。
- **UI quality — PASS**: 使用少量共享 CSS 变量和基础组件，不引入组件库；明确键盘、焦点、文字缩放、窗口尺寸和双平台视觉审查。

### Post-design gate

- **Maintainability — PASS**: Phase 1 将持久化、协议、截图和 IPC 边界分离，但不增加单实现接口或仓储层；协议差异通过有三个实际分支的枚举分派处理。
- **Testing — PASS**: 数据状态迁移、协议归一化、取消、历史事务与安全边界均有对应自动化验证入口；真实系统权限和显示器组合保留人工平台验证。
- **User experience — PASS**: IPC 契约统一了所有可见状态与恢复动作；多窗口共享同一 Rust 状态源，避免各窗口各自推断请求状态。
- **UI quality — PASS**: 界面结构、代表性尺寸和视觉证据在 quickstart 中可执行；模型输出首版按纯文本安全展示，避免富文本执行风险与额外依赖。

## Project Structure

### Documentation (this feature)

```text
specs/001-screenshot-ai-translation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── ipc.md
│   └── providers.md
└── tasks.md                 # 由 /speckit-tasks 后续生成
```

### Source Code (repository root)

```text
src/
├── main.tsx
├── App.tsx                  # 按 Tauri 窗口标签选择视图，不引入路由器
├── ipc.ts                   # 唯一的 Tauri command/channel 前端封装
├── i18n.ts                  # 简体中文资源及最小取词函数
├── styles.css               # 设计变量、布局、焦点和状态样式
├── components/
│   ├── Button.tsx
│   ├── Field.tsx
│   ├── EmptyState.tsx
│   └── ErrorNotice.tsx
└── views/
    ├── CaptureOverlay.tsx
    ├── Result.tsx
    ├── Onboarding.tsx
    ├── History.tsx
    ├── Prompts.tsx
    └── Settings.tsx

src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
│   └── default.json
├── migrations/
│   └── 0001_init.sql
├── src/
│   ├── main.rs
│   ├── lib.rs               # Tauri builder、托盘、单实例和插件初始化
│   ├── commands.rs          # 仅 IPC 边界和输入验证
│   ├── state.rs             # 单一 AppState 与活动请求取消句柄
│   ├── capture.rs           # 显示器快照、物理坐标映射、裁剪与 PNG 归一化
│   ├── analysis.rs          # 单请求状态机、Channel 输出和历史落库
│   ├── providers/
│   │   ├── mod.rs           # ProviderProtocol 枚举分派和统一事件
│   │   ├── openai.rs
│   │   ├── anthropic.rs
│   │   └── gemini.rs
│   ├── database.rs          # 连接、迁移与事务辅助
│   ├── history.rs           # 参数化查询、删除和再次提交数据读取
│   ├── credentials.rs       # keyring 的窄封装
│   ├── settings.rs          # 模型配置、提示词和应用设置
│   └── error.rs             # 可序列化、可脱敏的稳定错误码
└── tests/
    ├── provider_contracts.rs
    ├── history_integration.rs
    └── analysis_flow.rs

wdio.conf.ts
tests/
└── e2e/
    └── primary-flow.spec.ts
```

**Structure Decision**: 使用 create-tauri-app 的单项目结构。React 只处理本地多视图界面；所有有权限、秘密、文件、数据库和网络能力的操作留在 Rust。协议适配器按服务商分为三个文件，其余逻辑保持直接函数和枚举分派，不建立通用 provider SDK。

## Complexity Tracking

无宪章违规，不需要例外。
