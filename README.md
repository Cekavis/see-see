# See See

See See 是一个本地优先的 Windows/macOS 截图翻译桌面应用。它不执行 OCR，而是把用户框选的 PNG 截图和当前提示词直接发送到用户自己的多模态模型端点，并以纯文本流式展示结果。

## 功能

- 全局快捷键区域截图，支持物理像素坐标、负坐标和跨显示器选区。
- OpenAI Chat Completions、Anthropic Messages、Gemini GenerateContent 三类 API 协议。
- 预设端点和自定义端点；远程地址只允许 HTTPS，本机回环可使用 HTTP。
- 多模型配置、系统凭据存储、连接测试和手动模型 ID。
- 两个内置提示词以及提示词新增、编辑、复制、删除和切换。
- 完整本地历史：截图、缩略图、结果、失败状态、搜索、筛选、复制、再次提交、单条删除和全部清空。
- 首次引导、托盘/菜单栏、可配置快捷键、可选开机启动和脱敏日志导出。

应用不提供账号、额度、代理或中转服务。模型请求从本机直接发往用户配置的端点，相关费用由该端点账户承担。

## 开发环境

- Windows 10/11 或 macOS 14+
- Rust 1.95
- Node.js 24 LTS 或更新版本、npm
- 一个支持图片输入的兼容模型端点

```powershell
npm install
npm run tauri dev
```

首次启动时依次确认屏幕权限、模型配置和当前提示词。macOS 需要授予“屏幕录制”权限；Windows 通常无需额外授权。默认截图快捷键为 `CommandOrControl+Shift+X`，可在设置中修改。

## 数据与隐私

- 数据库文件名为 `see-see.sqlite3`，位于 Tauri `app_data_dir/app.seesee.desktop` 对应目录。Windows 通常位于 `%APPDATA%\app.seesee.desktop\`；macOS 通常位于 `~/Library/Application Support/app.seesee.desktop/`。
- API Key 不进入 SQLite，而是存入 Windows Credential Manager 或 macOS Keychain，服务名为 `See See`。
- 本地日志位于 Tauri `app_log_dir`：Windows 为 `%LOCALAPPDATA%\app.seesee.desktop\logs\`，macOS 为 `~/Library/Logs/app.seesee.desktop/`。
- 截图、提示词和模型输出不会用于遥测。开启历史时，它们保存在本机数据库；关闭“保存历史”后，新请求不写入历史。
- “导出诊断日志”只能通过原生保存对话框选择目标文件，导出前会遮盖认证头、API Key 参数和完整模型响应字段。

删除单条历史会级联删除对应图像；“清空全部历史”会删除全部历史和图像并压缩数据库。共享设备用户应关闭历史或在使用后清空。

## 安全边界

- 不允许 URL 内嵌用户名、密码或片段。
- 网络客户端禁止 HTTP 重定向，不提供跳过 TLS 验证的选项。
- 截图归一化为 PNG，最长边不超过 8000 像素，Base64 数据不超过 8 MiB。
- 模型输出通过 React 文本节点和 `<pre>` 渲染，不解释 Markdown/HTML，也不执行脚本或远程内容。
- 前端 Tauri capability 仅启用 `core:default`；文件写入只发生在应用数据目录或用户通过保存对话框选择的路径。

完整审计见 `specs/001-screenshot-ai-translation/validation/security.md`。

## 验证

```powershell
npm run typecheck
npm run lint
npm test
npm run test:e2e
npm run format:check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

`npm run test:e2e` 启动临时 Vite 服务，在 Chrome 中用页面加载前的窄 IPC mock 桥验证桌面壳层的关键命令顺序；真实 Rust 后端、窗口、快捷键和系统集成由 Rust 测试、`tauri dev`、平台人工验证与发布构建覆盖。

目前自动检查可在 Windows 执行；真实 macOS 权限、菜单栏、双屏和发布包仍必须在 macOS 14+ 设备上复验。详细矩阵见 `specs/001-screenshot-ai-translation/quickstart.md`。

## 打包

```powershell
npm run tauri build
```

Windows 生成安装包和可执行文件；macOS 发布应在 macOS 主机上生成 universal 包，并另行完成签名、公证和屏幕录制权限验证。仓库不包含 API Key、签名证书或公证凭据。
