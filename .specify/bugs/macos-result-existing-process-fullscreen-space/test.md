# Bug Test: macOS 旧进程的结果窗落到 Desktop Space

- **Slug**: macos-result-existing-process-fullscreen-space
- **Verified**: 2026-08-14
- **Fix**: ./fix.md
- **Status**: passed

## Scenario

1. 在 Desktop Space 启动 See See，保持进程驻留后台。
2. 打开 ChatGPT（或 Zen）并让其进入 macOS 原生全屏，形成独占 Space。
3. 在该全屏 Space 触发截图翻译并完成框选。
4. 观察结果窗口是否出现在当前全屏 Space。

## Result

结果窗口出现在当前全屏 Space，不再落到 Desktop Space。ChatGPT 与 Zen 两种全屏应用均验证通过。

## Notes

修复后 See See 以 accessory 身份运行，不再显示在 Dock；托盘图标、全局快捷键与主窗口均正常。
