# macOS 平台支持策略

See See 当前只支持 macOS 26 及以后版本。项目优先采用这些系统推荐的原生能力，不为更旧的 macOS 保留替代实现、运行时探测或降级路径。

## 登录时启动

- 主应用登录项使用 ServiceManagement 的 `SMAppService.mainAppService` 注册。
- 应用通过 `SMAppService.status` 区分已启用、未注册、需要用户批准和应用不可用。
- 需要批准时，界面提供前往“系统设置 > 通用 > 登录项与扩展”的操作。
- 不使用直接写入 `~/Library/LaunchAgents` 的方式，也不使用 AppleScript 创建旧式登录项。
- 升级后首次成功注册或关闭开机启动时，应用会清理旧版本可能留下的 LaunchAgent，避免重复启动。

`SMAppService` 要求应用经过代码签名并从完整的 `.app` 包运行。开发模式只能验证编译和状态映射；登录项的最终验收必须使用已签名并安装到 `/Applications` 的发布包。

## 维护原则

- `src-tauri/tauri.conf.json` 的 `minimumSystemVersion` 必须保持为 `26.0` 或更高。
- 新增 macOS 平台能力时，以当前 macOS 26+ SDK 推荐 API 为基线。
- 如果新 API 与旧系统不兼容，直接更新最低系统要求，不增加旧 API fallback。
- 发布验证至少覆盖最低支持版本和当前开发主机版本。
