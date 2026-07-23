# Tasks: See See 多模态截图翻译

**Input**: Design documents from `/specs/001-screenshot-ai-translation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: 行为变更必须先编写与其风险相称的自动化测试并确认测试在实现前失败。真实系统权限、多显示器和视觉质量使用记录在案的人工验证补充，不用脆弱模拟代替。

**Organization**: 任务按用户故事分组；Setup 和 Foundational 只包含所有故事共享或阻塞的工作。

## Format: `[ID] [P?] [Story] Description`

- **[P]**：在其前置任务完成后，可与同阶段其他 `[P]` 任务并行，且不修改相同文件
- **[Story]**：对应 spec.md 的用户故事
- 每项任务均包含需要创建或修改的明确路径

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 初始化单仓库 Tauri 桌面项目、工具链和测试入口。

- [X] T001 使用 Tauri 2 + React + TypeScript 模板初始化项目骨架，创建 `package.json`、`index.html`、`src/main.tsx`、`src/App.tsx`、`src-tauri/Cargo.toml`、`src-tauri/src/main.rs` 和 `src-tauri/src/lib.rs`
- [X] T002 [P] 在 `package.json` 和 `src-tauri/Cargo.toml` 添加 plan.md 指定的运行时、Tauri 官方插件、Rust 库和测试依赖，并配置 `dev`、`build`、`typecheck`、`lint`、`test`、`test:e2e` 脚本
- [X] T003 [P] 配置 TypeScript、Vite、ESLint、Prettier 和 Vitest，创建 `tsconfig.json`、`tsconfig.node.json`、`vite.config.ts`、`eslint.config.js`、`.prettierrc.json` 和 `vitest.config.ts`
- [X] T004 [P] 配置桌面烟雾测试入口，创建 `wdio.conf.ts` 和 `tests/e2e/.gitkeep`
- [X] T005 [P] 配置应用标识、窗口默认值和最小权限，更新 `src-tauri/tauri.conf.json` 并创建 `src-tauri/capabilities/default.json`

**Checkpoint**: 项目可安装依赖，前端、Rust 和测试命令均有稳定入口。

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 建立所有用户故事共享的数据、安全、协议、状态和 IPC 基础。

**⚠️ CRITICAL**: 本阶段完成前不得开始用户故事实现。

### Tests for foundational behavior

- [X] T006 [P] 为稳定错误码、错误脱敏、URL 安全规则和 API Key 不可序列化编写失败测试，创建 `src-tauri/tests/foundation.rs`
- [X] T007 [P] 按 `contracts/providers.md` 为三种请求体、认证头、模型列表、流式增量、空结果、超时和无自动重试编写失败测试，创建 `src-tauri/tests/provider_contracts.rs`
- [X] T008 [P] 为数据库 PRAGMA、初始设置、内置提示词、历史成功/失败原子写入及关闭历史不落库编写失败测试，创建 `src-tauri/tests/storage_foundation.rs`

### Implementation for shared foundation

- [X] T009 实现可序列化且默认脱敏的 `AppError`、稳定错误码与用户恢复动作，创建 `src-tauri/src/error.rs`
- [X] T010 创建 `src-tauri/migrations/0001_init.sql` 并在 `src-tauri/src/database.rs` 实现单连接初始化、版本迁移、外键、secure-delete、DELETE journal、FULL synchronous 和事务辅助
- [X] T011 实现唯一 `AppState`、capture/analysis 状态枚举、活动取消句柄和窗口状态，创建 `src-tauri/src/state.rs`
- [X] T012 实现 Windows Credential Manager/macOS Keychain 的窄封装和测试替身，创建 `src-tauri/src/credentials.rs`
- [X] T013 实现共享数据结构、应用快照、活动模型/提示词只读快照和默认设置读取，创建 `src-tauri/src/settings.rs`
- [X] T014 实现历史成功/失败记录的两表事务写入、缩略图生成和 `save_history` 守卫，创建 `src-tauri/src/history.rs`
- [X] T015 实现共享 HTTP client、严格端点拼接、PNG 限制/缩放、协议枚举、统一事件和错误映射，创建 `src-tauri/src/providers/mod.rs`
- [X] T016 [P] 实现 OpenAI Chat Completions 图片请求、模型列表和 SSE 文本解析，创建 `src-tauri/src/providers/openai.rs`
- [X] T017 [P] 实现 Anthropic Messages 图片请求、模型列表和 SSE 文本解析，创建 `src-tauri/src/providers/anthropic.rs`
- [X] T018 [P] 实现 Gemini GenerateContent 图片请求、模型列表和流式文本解析，创建 `src-tauri/src/providers/gemini.rs`
- [X] T019 按 `contracts/ipc.md` 建立命令注册表、共享输入验证和 `get_app_snapshot`/`open_view`/`quit_app` 基础命令，创建 `src-tauri/src/commands.rs`
- [X] T020 初始化数据库、AppState、日志及 global-shortcut、autostart、single-instance、clipboard-manager、opener、dialog 插件，更新 `src-tauri/src/lib.rs`
- [X] T021 [P] 创建简体中文语言资源、设计变量和可访问基础组件，新增 `src/i18n.ts`、`src/styles.css`、`src/components/Button.tsx`、`src/components/Field.tsx`、`src/components/EmptyState.tsx`、`src/components/ErrorNotice.tsx`
- [X] T022 实现类型化 command/channel 封装和按 Tauri 窗口标签选择视图的最小入口，更新 `src/ipc.ts`、`src/App.tsx` 和 `src/main.tsx`

**Checkpoint**: `cargo test` 的基础与 provider contract 测试通过；空窗口可以启动，前端无数据库、凭据或远程网络权限。

---

## Phase 3: User Story 1 - 快捷截图并获得模型解析结果 (Priority: P1) 🎯 Core MVP

**Goal**: 通过默认全局快捷键跨显示器框选区域，直接发送截图与当前提示词，并在可取消的结果浮窗中显示流式文本。

**Independent Test**: 通过测试夹具注入有效模型和提示词，从任意应用框选日文区域，验证无 OCR、单活动请求、流式结果、复制、置顶、取消和错误恢复。

### Tests for User Story 1 ⚠️

- [X] T023 [P] [US1] 为负坐标、混合缩放、反向拖动、跨屏裁剪、零尺寸取消、PNG 缩放和 8 MiB 限制编写失败测试，创建 `src-tauri/tests/capture_flow.rs`
- [X] T024 [P] [US1] 为单活动请求、Channel 快照/增量、唯一终止事件、取消不落历史、失败不自动重试和结果保存失败回退编写失败测试，创建 `src-tauri/tests/analysis_flow.rs`
- [X] T025 [P] [US1] 为 Pointer Capture、选区反馈、流式/非流式状态、置顶、复制全文、`Esc` 和关闭中取消编写失败测试，创建 `src/views/CaptureOverlay.test.tsx` 和 `src/views/Result.test.tsx`

### Implementation for User Story 1

- [X] T026 [US1] 实现最小多显示器冻结帧与跨 overlay Pointer Capture 实验，更新 `src-tauri/src/capture.rs` 和 `src/views/CaptureOverlay.tsx`
- [ ] T027 [US1] 在 Windows 混合 DPI 与 macOS 双屏上验证 T026 的负坐标和跨屏拖动；记录通过证据或仅替换输入跟踪层的修正决定到 `specs/001-screenshot-ai-translation/validation/capture-spike.md`
- [X] T028 [US1] 完成 xcap 显示器快照、物理坐标映射、跨屏裁剪拼接、PNG 归一化和 session 释放，更新 `src-tauri/src/capture.rs`
- [X] T029 [US1] 实现提交、流式累积、取消、终止状态、结果快照和历史写入协调，创建 `src-tauri/src/analysis.rs`
- [X] T030 [US1] 实现 `begin_capture`、帧读取、选区同步、完成/取消、attach/cancel/close analysis、复制和置顶命令，并注册默认快捷键及 overlay/result 窗口，更新 `src-tauri/src/commands.rs` 和 `src-tauri/src/lib.rs`
- [X] T031 [P] [US1] 完成跨显示器截图遮罩、拖动反馈和取消交互，更新 `src/views/CaptureOverlay.tsx`
- [X] T032 [P] [US1] 完成结果浮窗的处理中、流式、成功、失败、取消、复制和置顶状态，创建 `src/views/Result.tsx`
- [X] T033 [US1] 补齐截图与分析的前端 IPC/Channel 类型并接入窗口分发和共享样式，更新 `src/ipc.ts`、`src/App.tsx` 和 `src/styles.css`
- [ ] T034 [US1] 运行 US1 Rust/前端测试并按 independent test 完成一次真实截图验证，将命令、结果和未覆盖平台风险记录到 `specs/001-screenshot-ai-translation/validation/us1.md`

**Checkpoint**: 使用测试夹具模型时，US1 可以独立完成“快捷键 → 框选 → 流式结果 → 复制/取消”。

---

## Phase 4: User Story 2 - 配置自己的多模态模型服务 (Priority: P1) 🎯 Shippable MVP

**Goal**: 用户保存多个 OpenAI、Anthropic、Gemini 或对应自定义端点配置，安全保存凭据、获取/手填模型、连接测试并选择当前配置。

**Independent Test**: 创建三种预设协议和一个本机自定义配置，验证 HTTPS/回环规则、Key 不回传、模型列表失败回退、图片连接测试、切换与删除恢复。

### Tests for User Story 2 ⚠️

- [X] T035 [P] [US2] 为模型配置 CRUD、名称唯一、修改后重置测试状态、Key 写入/清除/删除回滚、active 约束和秘密脱敏编写失败测试，创建 `src-tauri/tests/model_config.rs`
- [X] T036 [P] [US2] 为模型列表、手动模型 ID、连接测试费用提示、加载/成功/分类错误和 Key 输入清空编写失败测试，创建 `src/views/Settings.model.test.tsx`

### Implementation for User Story 2

- [X] T037 [US2] 实现模型配置保存、编辑、删除、凭据补偿、active 选择和连接状态持久化，更新 `src-tauri/src/settings.rs` 和 `src-tauri/src/credentials.rs`
- [X] T038 [US2] 实现可选模型列表和极小 PNG 图片能力连接测试的协议分派，更新 `src-tauri/src/providers/mod.rs` 和 `src-tauri/src/settings.rs`
- [X] T039 [US2] 实现模型配置 IPC 命令与前端类型化封装，更新 `src-tauri/src/commands.rs` 和 `src/ipc.ts`
- [X] T040 [US2] 实现模型配置列表/表单、预设与自定义端点、模型列表/手填、连接测试、active 切换、删除确认和错误恢复，创建 `src/views/Settings.tsx`
- [X] T041 [US2] 验证远程 HTTP 拒绝、本机 HTTP 允许、错误 Key、模型不支持图片和删除凭据路径，并记录数据库/日志无明文 Key 的证据到 `specs/001-screenshot-ai-translation/validation/us2.md`

**Checkpoint**: Setup + Foundation + US1 + US2 构成可交付 BYOK MVP。

---

## Phase 5: User Story 3 - 创建和切换提示词预设 (Priority: P2)

**Goal**: 用户管理多个普通文本提示词，并让当前预设决定后续截图和历史再次提交的分析目标。

**Independent Test**: 使用两个内置预设处理同一截图，再新增、复制、编辑、删除和切换预设，验证输出要求和当前状态变化。

### Tests for User Story 3 ⚠️

- [X] T042 [P] [US3] 为内置提示词、名称/正文限制、复制唯一名称、删除 active 置空和快照不可变编写失败测试，创建 `src-tauri/tests/prompt_presets.rs`
- [X] T043 [P] [US3] 为提示词加载、空态、新增、编辑、复制、删除确认、active 切换和键盘焦点编写失败测试，创建 `src/views/Prompts.test.tsx`

### Implementation for User Story 3

- [X] T044 [US3] 实现提示词 CRUD、复制命名、active 选择和删除置空命令，更新 `src-tauri/src/settings.rs` 和 `src-tauri/src/commands.rs`
- [X] T045 [US3] 实现提示词 IPC 封装与完整管理界面，更新 `src/ipc.ts` 并创建 `src/views/Prompts.tsx`
- [X] T046 [US3] 在请求启动时固定当前提示词名称/正文快照，并在无 active 提示词时阻止提交，更新 `src-tauri/src/analysis.rs`
- [ ] T047 [US3] 使用两个内置提示词处理同一截图并完成 CRUD/空态验证，将结果记录到 `specs/001-screenshot-ai-translation/validation/us3.md`

**Checkpoint**: US3 独立可管理提示词；US1 自动使用当前预设且不提供语言选择器或变量系统。

---

## Phase 6: User Story 4 - 查看和管理完整本地历史 (Priority: P2)

**Goal**: 提供本地历史列表与详情、结果搜索、提示词筛选、原图、复制、再次提交、删除、清空和关闭历史。

**Independent Test**: 生成成功和失败记录，在 10,000 条夹具中搜索/筛选目标，查看原图、复制、使用当前配置再次提交、删除并确认关闭历史后不持久化。

### Tests for User Story 4 ⚠️

- [X] T048 [P] [US4] 为 cursor 分页、转义 LIKE、提示词/状态筛选、二进制图片读取、级联删除、清空后 VACUUM 和损坏图片错误编写失败测试，更新 `src-tauri/tests/history_integration.rs`
- [X] T049 [P] [US4] 创建 10,000 条长中文/日文记录的 release 基准并断言打开、搜索和筛选目标，创建 `src-tauri/tests/history_benchmark.rs`
- [X] T050 [P] [US4] 为历史加载/空态/无结果、列表/详情、失败详情、复制、再次提交、单条删除和全部清空编写失败测试，创建 `src/views/History.test.tsx`

### Implementation for User Story 4

- [X] T051 [US4] 实现稳定 cursor 分页、参数化结果搜索、提示词/状态筛选、详情/图片读取、级联删除和清空 VACUUM，更新 `src-tauri/src/history.rs`
- [X] T052 [US4] 实现历史原图使用当前模型和提示词再次提交，并补齐成功、失败、取消和关闭历史的持久化边界，更新 `src-tauri/src/history.rs` 和 `src-tauri/src/analysis.rs`
- [X] T053 [US4] 实现 history/query/image/resubmit/delete/clear 与 `set_save_history` IPC 命令和前端封装，更新 `src-tauri/src/commands.rs` 和 `src/ipc.ts`
- [X] T054 [P] [US4] 实现历史搜索筛选、稳定分页、缩略图列表、详情、失败态、复制、再次提交和删除确认，创建 `src/views/History.tsx`
- [X] T055 [P] [US4] 在设置界面加入默认开启的“保存历史”开关和本地截图隐私说明，更新 `src/views/Settings.tsx` 和 `src-tauri/src/settings.rs`
- [X] T056 [US4] 运行 10,000 条基准、关闭历史重启检查、删除/清空与 secure-delete 检查，将结果记录到 `specs/001-screenshot-ai-translation/validation/us4.md`

**Checkpoint**: US4 可用测试夹具独立浏览历史；与 US1 集成后支持真实记录和再次提交。

---

## Phase 7: User Story 5 - 在桌面后台可靠运行 (Priority: P3)

**Goal**: 应用通过 Windows 托盘/macOS 菜单栏常驻，提供首次引导、权限恢复、唯一窗口、快捷键配置、开机启动、日志导出和明确退出。

**Independent Test**: 首次启动完成权限、模型和提示词检查；关闭窗口后快捷键仍可用；托盘/菜单栏可打开各视图；快捷键冲突不破坏旧绑定；只有“退出”结束进程。

### Tests for User Story 5 ⚠️

- [X] T057 [P] [US5] 为快捷键先注册后替换、冲突回滚、autostart 状态同步、单实例窗口聚焦、退出取消活动请求和日志脱敏编写失败测试，创建 `src-tauri/tests/desktop_lifecycle.rs`
- [X] T058 [P] [US5] 为首次引导步骤、权限拒绝恢复、未配置时阻止完成、开机启动默认关闭和桌面设置状态编写失败测试，创建 `src/views/Onboarding.test.tsx` 和 `src/views/Settings.desktop.test.tsx`

### Implementation for User Story 5

- [X] T059 [US5] 实现 single-instance、系统托盘/macOS 菜单栏、唯一窗口复用、关闭隐藏、明确退出和活动请求清理，更新 `src-tauri/src/lib.rs`
- [X] T060 [US5] 实现快捷键原子替换、冲突恢复和 autostart 系统状态同步，更新 `src-tauri/src/settings.rs` 和 `src-tauri/src/lib.rs`
- [X] T061 [US5] 实现屏幕权限状态、打开 macOS 系统设置、onboarding 完成条件和应用快照命令，更新 `src-tauri/src/capture.rs`、`src-tauri/src/commands.rs` 和 `src-tauri/src/settings.rs`
- [X] T062 [US5] 实现无遥测的本地脱敏日志、原生保存对话框导出和导出失败恢复，更新 `src-tauri/src/lib.rs`、`src-tauri/src/commands.rs` 和 `src-tauri/src/error.rs`
- [X] T063 [US5] 实现简体中文首次引导及快捷键、开机启动、日志导出桌面设置，创建 `src/views/Onboarding.tsx` 并更新 `src/views/Settings.tsx`、`src/ipc.ts` 和 `src/App.tsx`
- [ ] T064 [US5] 在 Windows 与 macOS 验证首次权限、托盘/菜单栏、窗口关闭、重复启动、快捷键冲突、开机启动和退出，将证据记录到 `specs/001-screenshot-ai-translation/validation/us5.md`

**Checkpoint**: 所有五个用户故事在目标桌面平台形成完整产品流程。

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: 完成跨故事验证、可访问性、视觉质量、安全、准确度与发布构建。

- [X] T065 [P] 按 IPC mock + 桌面运行模式实现“配置模型 → 截图结果 → 历史查看”的关键烟雾测试，创建 `tests/e2e/primary-flow.spec.ts` 并完善 `wdio.conf.ts`
- [X] T066 [P] 编写开发、运行、隐私、数据位置、平台权限和验证命令说明，创建 `README.md`
- [X] T067 [P] 审计凭据、端点重定向、日志、模型输出纯文本渲染、Tauri capabilities 和任意路径写入，记录发现与修复证据到 `specs/001-screenshot-ai-translation/validation/security.md`
- [ ] T068 在 quickstart.md 的完整单屏/双屏/负坐标/混合 DPI 矩阵上验证 Windows 与 macOS，记录设备、缩放和截图证据到 `specs/001-screenshot-ai-translation/validation/cross-platform.md`
- [ ] T069 在 1024×720、1440×900、360×240 和 200% 文字缩放下完成键盘、焦点、对比度、状态一致性和双平台人工视觉审查，记录到 `specs/001-screenshot-ai-translation/validation/visual-review.md`
- [ ] T070 [P] 准备 100 张无敏感信息的日文截图与固定 OCR 基线清单，创建 `tests/fixtures/accuracy/manifest.json`，执行双语评审并记录 85% 可用率及高于基线 10 个百分点的结果到 `specs/001-screenshot-ai-translation/validation/accuracy.md`
- [ ] T071 在 Windows 与 macOS 运行 typecheck、lint、Vitest、WebdriverIO、fmt、clippy、cargo test 和 Tauri build，并记录命令、版本、结果与已知无关失败到 `specs/001-screenshot-ai-translation/validation/builds.md`
- [ ] T072 按 `specs/001-screenshot-ai-translation/quickstart.md` 完成最终端到端复验，确认所有规格成功标准和宪章质量门禁后记录交付结论到 `specs/001-screenshot-ai-translation/validation/final.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 — Setup**: 无依赖；T001 完成后 T002–T005 可并行。
- **Phase 2 — Foundational**: 依赖 Setup；阻塞所有用户故事。T006–T008 必须先失败，随后完成 T009–T022。
- **Phase 3 — US1**: 依赖 Foundational；T027 是跨平台截图实现门禁。
- **Phase 4 — US2**: 依赖 Foundational；可与 US1 开发并行，但可交付 MVP 需要 US1 + US2 都完成。
- **Phase 5 — US3**: 依赖 Foundational；可与 US1/US2 并行。
- **Phase 6 — US4**: 查询和界面可在 Foundational 后开始；完整“再次提交”依赖 US1 的 analysis engine。
- **Phase 7 — US5**: 依赖 US1 的截图流程和 US2 的有效模型配置；引导可使用 Foundational 中的内置提示词，不依赖 US3 管理界面。
- **Phase 8 — Polish**: 依赖计划纳入发布范围的所有用户故事；T065、T066、T067、T070 可先行准备，T068、T069、T071、T072 在功能完成后执行。

### User Story Dependency Graph

```text
Setup
  -> Foundational
       -> US1 Screenshot & Result -------> US4 Resubmit
       -> US2 Model Configuration -------> US5 Onboarding/Desktop Lifecycle
       -> US3 Prompt Management
       -> US4 History Query/UI

US1 + US2 = Shippable BYOK MVP
US1 + US2 + US3 + US4 + US5 = v1 feature complete
```

### Within Each User Story

1. 先编写标记为 Tests 的任务并确认在实现前失败。
2. 完成后端领域/持久化逻辑，再暴露 IPC。
3. 完成界面和窗口集成。
4. 运行故事自动测试并记录独立人工验证。
5. 故事 checkpoint 通过后才能视为完成。

## Parallel Opportunities

- T002–T005 可在 T001 后并行。
- T006–T008、T016–T018、T021 可按文件并行。
- Foundational 完成后，US1、US2 和 US3 可由不同开发者并行；US4 的查询/UI 部分也可并行，但再次提交等待 US1。
- 每个故事的 `[P]` 测试任务可以并行，随后共同驱动实现。
- T031/T032、T054/T055 以及 T065/T066/T067/T070 修改不同文件，可并行。

## Parallel Example: User Story 1

```text
T023: src-tauri/tests/capture_flow.rs
T024: src-tauri/tests/analysis_flow.rs
T025: src/views/CaptureOverlay.test.tsx + src/views/Result.test.tsx

测试失败后并行：
T031: src/views/CaptureOverlay.tsx
T032: src/views/Result.tsx
```

## Parallel Example: User Story 2

```text
T035: src-tauri/tests/model_config.rs
T036: src/views/Settings.model.test.tsx
```

## Parallel Example: User Story 3

```text
T042: src-tauri/tests/prompt_presets.rs
T043: src/views/Prompts.test.tsx
```

## Parallel Example: User Story 4

```text
T048: src-tauri/tests/history_integration.rs
T049: src-tauri/tests/history_benchmark.rs
T050: src/views/History.test.tsx

IPC 完成后并行：
T054: src/views/History.tsx
T055: src/views/Settings.tsx + src-tauri/src/settings.rs
```

## Parallel Example: User Story 5

```text
T057: src-tauri/tests/desktop_lifecycle.rs
T058: src/views/Onboarding.test.tsx + src/views/Settings.desktop.test.tsx
```

## Implementation Strategy

### Shippable MVP First

1. 完成 Phase 1：Setup。
2. 完成 Phase 2：Foundational。
3. 完成 Phase 3：US1 截图与结果。
4. 完成 Phase 4：US2 BYOK 模型配置。
5. **STOP AND VALIDATE**：执行 T034、T041 和最小跨平台检查；此时用户可以自行配置模型并完成截图分析。

### Incremental Delivery

1. US1 + US2：可交付 BYOK 截图分析 MVP。
2. US3：加入多提示词工作流。
3. US4：加入完整本地历史与再次提交。
4. US5：完成首次引导、托盘/菜单栏和桌面生命周期。
5. Phase 8：完成 v1 安全、准确度、视觉和双平台发布门禁。

### Minimality Rules

- 不增加后端服务、账号、云同步、OCR、Markdown 渲染器、路由器、全局状态库、ORM 或 provider SDK。
- 不为未来协议建立动态插件系统；仅使用三个已有协议分支。
- 10,000 条历史基准未失败前，不增加 FTS 或外部搜索索引。
- 不以删除测试、缩小断言范围或跳过平台验证换取通过。

## Notes

- `[P]` 仅表示在其前置任务完成后文件互不冲突；同一文件的后续任务必须按 ID 顺序执行。
- 用户故事标签用于规格追踪，Setup、Foundational 和 Polish 不使用故事标签。
- 任务完成时应在对应 validation 文档记录实际命令和证据；不得填写未运行的结果。
- 不自动提交 Git commit；只有用户明确要求时才提交。
