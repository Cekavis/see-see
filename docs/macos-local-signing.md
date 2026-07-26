# macOS 少量设备自签名发布

See See 的屏幕录制权限由 macOS TCC 绑定到代码签名身份。仓库固定使用名为 `See See Local Release` 的自签名证书；只要证书私钥和 bundle identifier `app.seesee.desktop` 不变，新版本就能沿用同一应用身份。

此方案只适合自己控制的少量 Mac。它不提供 Apple 公证，不能替代面向公众发布所需的 Developer ID。

## 1. 在构建 Mac 上创建证书

只需创建一次：

1. 打开“钥匙串访问”，选择“登录”钥匙串。
2. 从菜单选择“钥匙串访问 > 证书助理 > 创建证书”。
3. 名称填写 `See See Local Release`，身份类型选择“自签名根证书”，证书类型选择“代码签名”。
4. 勾选“让我覆盖默认值”，继续。
5. 序列号使用一个非零整数；有效期可填写 `3650` 天。其余名称字段可以保持默认。
6. 密钥对使用 RSA、2048 位；Key Usage 保留数字签名，Extended Key Usage 保留代码签名。
7. 将证书保存到“登录”钥匙串。
8. 在钥匙串中双击该证书，展开“信任”，把“代码签名”设为“始终信任”。关闭窗口并按系统提示确认登录密码或 Touch ID。

证书名称必须完全一致。终端验证：

```bash
security find-identity -v -p codesigning
```

输出应包含 `See See Local Release` 和 `1 valid identities found`。如果没有，检查证书是否与私钥成对出现，以及“代码签名”信任设置。

## 2. 构建并校验

```bash
npm run tauri build
npm run verify:macos-signature
```

macOS 平台配置会强制 Tauri 使用 `See See Local Release`。证书缺失时构建会失败，不会再静默生成会破坏 TCC 权限的 ad-hoc 升级包。

校验脚本还会拒绝以下产物：

- `Signature=adhoc`；
- designated requirement 仅绑定本次构建 CDHash；
- 证书名称或 bundle identifier 不匹配；
- Info.plist 未绑定或应用资源未封装。

DMG 位于 `src-tauri/target/release/bundle/dmg/`。

## 3. 首次从 ad-hoc 版本迁移

从旧的 ad-hoc 版本切换到自签名版本时，代码身份必然变化一次：

1. 完全退出 See See，包括菜单栏中的进程。
2. 从 DMG 把新版复制到 `/Applications/See See.app`，替换旧版本。
3. 如果 macOS 阻止首次打开，在 Finder 中按住 Control 点击应用并选择“打开”；或前往“系统设置 > 隐私与安全性”，确认“仍要打开”。不要关闭系统的安全检查。
4. 前往“系统设置 > 隐私与安全性 > 屏幕与系统音频录制”，删除旧的 See See 记录。
5. 添加当前 `/Applications/See See.app` 并启用权限，然后完全退出并重新打开应用。
6. 确认应用显示“屏幕权限已就绪”，并完成一次真实截图。

这应是最后一次因本次签名迁移而重新授权。

## 4. 后续升级

1. 始终在持有同一 `See See Local Release` 私钥的构建 Mac 上生成包。
2. 完全退出旧版，再用新版本替换 `/Applications/See See.app`。
3. 不要删除屏幕录制权限记录；启动后确认权限仍为就绪。
4. 如再次要求授权，先运行 `npm run verify:macos-signature`，不要直接删除 TCC 记录，以免掩盖签名回归。

可比较两个版本的 designated requirement；内容应保持一致且不应以 `cdhash` 开头：

```bash
codesign -d --requirements - "/Applications/See See.app" 2>&1
```

## 5. 配置其他自用 Mac

目标 Mac 只需要信任公开证书，不需要私钥：

1. 在构建 Mac 的钥匙串中选中 `See See Local Release` 证书本身，不要选私钥。
2. 导出为 `.cer`，通过可信渠道传到目标 Mac。
3. 在目标 Mac 双击 `.cer` 导入“登录”钥匙串。
4. 双击证书，展开“信任”，把“代码签名”设为“始终信任”。
5. 安装签名后的 See See，并按“首次从 ad-hoc 版本迁移”完成一次授权。

只有需要在另一台 Mac 上构建时才导出证书和私钥为加密的 `.p12`。为 `.p12` 设置强密码，通过可信渠道传输，导入完成后删除传输副本。不要把 `.cer`、`.p12`、私钥、密码或任何签名材料提交到仓库。

## 限制

- 自签名包不能提交 Mac App Store，也不能通过 Apple 公证。
- 从浏览器或聊天工具下载后，目标 Mac 可能仍要求用户首次确认打开。
- 证书到期、丢失、重新创建，或更改 bundle identifier 后，需要重新建立权限。
- 面向不受控制的用户分发时，应改用同一 Apple Developer Team 的 Developer ID Application 证书和公证流程。
