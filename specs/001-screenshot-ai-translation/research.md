# Research: See See 多模态截图翻译

## 1. 桌面技术栈

**Decision**: 使用 Tauri 2.x + React 19.2 + TypeScript + Vite，Node.js 使用 24 LTS，Rust 固定为当前稳定工具链 1.95。

**Rationale**: Tauri 原生支持多窗口、系统托盘、全局快捷键、开机启动、单实例和严格 capability 权限；Rust 后端适合处理截图、SQLite、系统凭据与流式 HTTP。React 用于多个有状态表单、历史主从视图和结果流更新，但只使用 Hooks，不增加路由器或状态库。Node 官方将 24 标记为 LTS，React 官方当前主版本为 19.2。

**Alternatives considered**:

- Electron：桌面 API 成熟，但需要更大的常驻运行时；本项目仍需原生模块完成系统凭据和精确截图，没有减少核心复杂度。
- 分别使用 WinUI 与 Swift/AppKit：平台体验最佳，但会形成两套产品逻辑、测试和发布流程，不符合当前规模。
- Tauri + 原生 TypeScript DOM：依赖更少，但历史、设置、引导和流式结果的状态同步会产生更多手写 DOM 生命周期代码；React 是此处最小的可维护 UI 层。

**Sources**:

- [Tauri commands and channels](https://v2.tauri.app/develop/calling-rust/)
- [Tauri global shortcut](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri system tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri capabilities](https://v2.tauri.app/security/capabilities/)
- [Node.js releases](https://nodejs.org/en/about/previous-releases)
- [React versions](https://react.dev/versions)

## 2. 截图与跨显示器坐标

**Decision**: 使用 xcap 获取每个显示器的物理像素快照与几何信息；截图开始时先冻结所有显示器画面，再为每个显示器创建一个无边框置顶 overlay。开始拖动的 overlay 使用 Pointer Capture 持续报告全局选区，Rust 按每个显示器的物理矩形广播选区并在结束后裁剪、拼接为单张 PNG。

**Rationale**: xcap 当前直接支持 Windows 与 macOS 的显示器和区域截图。按物理像素保存每个显示器矩形，能够在负坐标、不同缩放比例和高像素密度下避免把 CSS 坐标误当截图像素。先截图再显示遮罩可防止遮罩进入结果图，也能让跨屏选区以冻结画面一致呈现。

**Alternatives considered**:

- 单个跨越虚拟桌面的 WebView：混合 DPI 环境下单一窗口缩放因子无法可靠代表所有显示器。
- 调用系统截图工具：macOS 可行，但 Windows 的系统剪裁入口与完成通知不稳定，难以提供一致取消、历史和错误状态。
- 自写 Windows/macOS 截图 API：控制力最高，但重复 xcap 已覆盖的底层代码，扩大平台维护面。

**Implementation gate**: 在实现截图主流程前先完成一个最小平台实验，验证 Pointer Capture 跨 overlay 的拖动、负坐标和 Windows 混合 DPI；失败时仅替换 overlay 输入跟踪层，不替换 xcap、坐标模型或后续裁剪逻辑。

**Sources**:

- [xcap repository and examples](https://github.com/nashaofu/xcap)
- [xcap crate documentation](https://docs.rs/xcap/latest/xcap/)

## 3. 多模态协议兼容与流式归一化

**Decision**: Rust 使用一个共享 `reqwest::Client` 和三个直接 JSON 适配器，不引入服务商 SDK。OpenAI 协议调用 Chat Completions；Anthropic 调用 Messages；Gemini 调用 GenerateContent/streamGenerateContent。三个适配器统一输出 `Started`、`Delta`、`Completed`、`Failed` 事件，通过 Tauri Channel 发送给结果窗口。

**Rationale**: 三个协议都有明确的图片输入与流式能力。Chat Completions 在 OpenAI 官方仍支持图片分析，同时比 Responses API 更广泛地被 OpenAI-compatible 自定义端点实现。Gemini 官方当前推荐 Interactions API，但 GenerateContent 仍是稳定核心 API，并提供内联图片和模型列表，适合自定义 Gemini 端点与一次性图片任务。统一直接 HTTP 可避免引入三个大型 SDK及其不同流式抽象。

**Alternatives considered**:

- 服务商官方 SDK：每个 SDK 都带来独立类型、重试、默认行为和升级节奏；应用只需要一个图片、一段提示词和文本流。
- 统一第三方 AI SDK：会缩小可控的自定义端点行为，并引入本项目不需要的工具调用、会话和代理抽象。
- OpenAI Responses / Gemini Interactions：官方新能力更多，但当前产品不需要会话、工具或复合输出；首版优先兼容面和请求体最小化。

**Protocol details**:

- 图片统一为 base64 PNG；单张图片编码后超过 8 MiB 或任一边超过 8,000 px 时按比例缩小后重新编码，避免超过三类协议的共同安全范围。
- OpenAI 与 Anthropic 使用 SSE；Gemini 使用其流式 HTTP 响应。解析器只提取可见文本增量，不暴露供应商原始事件给界面。
- 连接测试发送一个极小 PNG 和固定短提示词，验证认证、模型图片能力和文本输出；界面明确说明测试可能产生极小调用费用。
- 模型列表是可选能力；失败时返回 `models_unavailable`，不阻止手动模型标识和连接测试。

**Sources**:

- [OpenAI images and vision](https://developers.openai.com/api/docs/guides/images-vision)
- [Anthropic vision](https://platform.claude.com/docs/en/build-with-claude/vision)
- [Anthropic streaming](https://platform.claude.com/docs/en/build-with-claude/streaming)
- [Gemini GenerateContent reference](https://ai.google.dev/api/generate-content)
- [Gemini model listing](https://ai.google.dev/api/models)

## 4. 本地数据与搜索

**Decision**: 使用 rusqlite 直接管理一个 bundled SQLite 数据库。历史元数据和结果文本在 `history_entries`，原图与缩略图在独立的 `history_images` BLOB 表，通过外键级联实现同一事务中的写入和删除。10,000 条目标规模内，结果文本搜索使用转义后的参数化 `LIKE`，并为时间、状态和提示词筛选建立普通索引。

**Rationale**: 把图片放在独立 BLOB 表可使列表查询不读取图片页，同时避免数据库记录与外部文件之间的孤儿、部分写入和敏感文件删除失败。10,000 条桌面历史的规模无需服务进程、ORM 或全文索引；顺序文本扫描的实现和删除语义更直接，并由性能基准决定是否需要升级。

**Alternatives considered**:

- 图片文件 + SQLite 元数据：节省数据库体积，但跨文件系统与数据库的原子写入、删除和崩溃恢复需要额外清理状态机。
- Tauri SQL 插件：前端会获得不必要的数据库权限，且仍需 Rust 保证历史原子性。
- ORM/仓储层：数据模型简单，直接 SQL 与小型迁移函数更清楚。
- FTS5：中文日文子串搜索更快，但 FTS5 影子表即使启用 `secure_delete` 仍可能留下删除痕迹；当前规模下不值得增加隐私和同步复杂度。

**Known ceiling**: 如果 10,000 条基准中参数化 `LIKE` 超过 2 秒，再评估不保存敏感全文副本的索引方案；如果图片 BLOB 增长到多 GB 后出现可测量的 VACUUM 或启动问题，再迁移图片到内容寻址文件存储。首版不为未出现的规模问题增加双存储协议。

**Deletion settings**: 每个连接启用 `foreign_keys=ON`、`secure_delete=ON`、`journal_mode=DELETE`、`synchronous=FULL` 和 `trusted_schema=OFF`。全部清空后执行 `VACUUM` 释放页面；产品仍明确说明未提供应用级加密，无法承诺对底层磁盘取证恢复的防护。

**Sources**:

- [SQLite FTS5](https://www.sqlite.org/fts5.html)
- [SQLite secure_delete and journal pragmas](https://www.sqlite.org/pragma.html)
- [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)

## 5. 凭据、网络和渲染安全

**Decision**: API Key 用 keyring 写入 Windows Credential Manager 或 macOS Keychain，数据库只保存不可逆推出秘密的凭据引用和 `has_api_key` 状态。所有外部 HTTP 由 Rust 发起；前端不持有 Key、不访问远程 URL。模型结果按纯文本 `white-space: pre-wrap` 展示。

**Rationale**: keyring 提供两目标平台的原生凭据存储。Rust 边界可统一执行 URL、TLS、超时、错误脱敏和日志过滤。纯文本满足选择、复制和保留空行的核心需求，并彻底避免模型输出中的 HTML、脚本、远程图片或 Markdown 链接执行。

**Alternatives considered**:

- Tauri Stronghold：提供加密保险库，但用户已选择操作系统凭据存储；增加主密码或保险库生命周期没有价值。
- 前端直接调用模型 API：会把 API Key 暴露给 WebView 状态和开发工具，并分散协议与日志规则。
- Markdown 渲染器：视觉更丰富，但用户提示词已经控制输出格式；首版没有值得承担的解析和净化依赖。

**Sources**:

- [keyring crate](https://docs.rs/keyring/latest/keyring/)
- [Tauri capabilities](https://v2.tauri.app/security/capabilities/)

## 6. 状态管理与多窗口通信

**Decision**: Rust `AppState` 是活动截图、活动分析、当前配置和持久化访问的唯一权威。普通 CRUD 使用 Tauri commands；分析流使用 Tauri Channel；跨窗口只发送小型状态变更事件。前端每个窗口使用局部 React state，不增加 Redux、Zustand 或路由器。

**Rationale**: 单活动请求约束让一个取消句柄和状态枚举足够。Tauri 官方将 Channel 推荐用于流式 HTTP 数据；大型截图通过二进制 IPC 响应按需读取，不放入 JSON 事件。

**Alternatives considered**:

- 全部使用全局事件：事件不具备返回值和强类型，流丢失或窗口晚加入时更难恢复。
- 前端全局状态库：多个 Tauri 窗口并不共享同一个 JavaScript 运行时，仍需要后端权威状态。
- 内部消息总线：单请求、少量窗口不需要额外抽象。

**Source**: [Tauri commands, binary responses and channels](https://v2.tauri.app/develop/calling-rust/)

## 7. 验证策略

**Decision**: Rust 测试覆盖 URL 安全、协议请求/流解析、错误归一化、状态迁移和 SQLite 事务；前端用 Vitest/Testing Library 覆盖所有界面状态、键盘和焦点；用 WebdriverIO Tauri service 保留一个设置到结果/历史的桌面烟雾测试。真实全局快捷键、屏幕权限、多显示器和视觉质量在 Windows/macOS 人工验证。

**Rationale**: 纯逻辑和边界可以快速自动验证；真实屏幕、权限弹窗和混合显示器组合依赖平台环境，不应通过脆弱的模拟测试伪装覆盖。Tauri 官方 WebDriver 路径当前可在 Windows、Linux 和 macOS 运行。

**Alternatives considered**:

- 只做单元测试：无法覆盖 IPC 和多窗口装配。
- 全量 GUI 自动化：屏幕权限、系统托盘和真实显示器依赖使测试成本远高于首版价值。

**Sources**:

- [Tauri WebDriver testing](https://v2.tauri.app/develop/tests/webdriver/)
- [Vitest guide](https://vitest.dev/guide/)
