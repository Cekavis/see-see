# Bug Assessment: 模型配置按钮缺少可见反馈

- **Slug**: model-config-buttons-inert
- **Created**: 2026-07-25
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 在模型配置界面，三个按钮点击都没有任何效果，请你调查修复

## Symptom

在模型配置编辑器底部点击“获取模型列表”“测试连接”或“保存配置”后，界面看起来没有响应。预期按钮立即显示进行中状态，并在按钮附近呈现成功或失败反馈。

## Reproduction

1. 打开主窗口并进入“模型”栏目。
2. 在 1024×720 窗口中滚动到模型编辑器底部。
3. 点击任一操作按钮，例如在必填字段为空时点击“保存配置”。
4. React 点击处理函数和 IPC 调用均会执行，但新增的反馈位于编辑器上方、粘性标题栏之后，用户当前视口中看不到反馈。

## Suspected Code Paths

- `src/views/Settings.tsx:91` — 共享的错误与成功反馈渲染在 `.section-split` 之前，与三个操作按钮相距整个编辑器高度。
- `src/views/Settings.tsx:190` — 三个按钮会更新 `busy`、`error` 或 `notice`，但按钮文案没有表示进行中状态。
- `src/styles.css:319` — 粘性标题栏占据视口顶部 88px 且 `z-index: 5`；滚动到操作区后，顶部反馈会被标题栏覆盖。
- `src/views/Settings.model.test.tsx` — 现有测试只断言服务调用及最终状态，未约束操作中的可见反馈及反馈与编辑器的相对位置。

## Root Cause Hypothesis

高置信度：三个按钮的事件处理和 IPC 调用本身正常，问题是共享反馈位置与粘性布局不兼容。在真实浏览器的 1280×720 视口中滚动到按钮后，反馈矩形位于 `top: 20px` 到 `65px`，而粘性标题栏覆盖 `top: 0` 到 `88px`，因此反馈被完全遮挡。按钮仅通过轻微的禁用透明度表示进行中，快速失败时几乎不可察觉，共同造成“点击无效果”的观感。

## Proposed Remediation

**Preferred**: 将模型操作的错误/成功反馈移动到模型配置编辑器内、紧邻费用提示和按钮行；三个按钮在对应请求进行时显示明确的“正在…”文案，并在任一模型操作进行时禁用并发操作。每次发起新操作时清除旧的错误和成功消息，避免旧状态掩盖当前结果。

**Files likely to change**:

- `src/views/Settings.tsx`
- `src/views/Settings.model.test.tsx`
- `src/styles.css`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 断言三个操作按钮发起请求后立即显示各自的进行中文案，并阻止并发操作。
- 断言错误/成功反馈属于“模型配置编辑器”区域，而不是粘性标题栏下方的页面级区域。
- 保留现有保存、连接失败和删除配置行为测试。

## Risks & Considerations

- 反馈移动后，初始配置列表加载失败也会显示在编辑器内；该位置仍然可见且不影响错误内容。
- 禁止并发模型操作会改变此前可同时发起多个请求的行为，但可避免响应乱序覆盖状态。
- 无 IPC 或数据格式变更。

## Open Questions

- 无。
