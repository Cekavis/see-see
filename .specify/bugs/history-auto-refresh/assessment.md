# Bug Assessment: 历史页不会自动显示新记录

- **Slug**: history-auto-refresh
- **Created**: 2026-08-11
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 历史页需要自动刷新，现在需要切出去再切回来才会显示新内容

## Symptom

历史页保持打开时，后台分析完成并保存的新记录不会出现在列表中。用户切换到其他设置栏目再切回历史页后，组件重新挂载并查询数据，新记录才会显示。

## Reproduction

1. 打开设置窗口并进入历史页。
2. 在历史页保持挂载的情况下完成一次会保存历史记录的分析。
3. 观察历史列表没有显示新记录；切换到其他栏目再返回历史页后，新记录出现。

## Suspected Code Paths

- `src/views/History.tsx:166` — 列表查询只由初次挂载、用户搜索、删除和清空操作触发，没有订阅后台新增记录。
- `src/views/SettingsShell.tsx:244` — 切换离开历史栏目会卸载 `History`，返回时重新挂载，从而触发初次查询。
- `src-tauri/src/analysis.rs:355` — 分析完成后写入历史数据库，但保存成功后没有向其他窗口发送历史数据已变化的通知。

## Root Cause Hypothesis

置信度：高。历史写入发生在 Rust 后台任务中，而历史列表的 React 状态仅在组件挂载或本页操作时更新。写入路径和展示路径之间缺少失效事件，所以已挂载页面会一直保留旧状态。

## Proposed Remediation

**Preferred**: 分析结果成功写入历史数据库后，通过现有 Tauri 全局事件机制发出 `history-updated` 通知。历史组件在挂载期间订阅该事件，收到通知后使用当前筛选条件重新查询第一页，并在卸载时解除订阅。

轮询可以达到类似效果，但会产生持续的无效 IPC/数据库查询，并引入刷新间隔；事件通知更直接且只在数据确实变化时工作。

**Files likely to change**:

- `src-tauri/src/analysis.rs`
- `src/views/History.tsx`
- `src/views/History.test.tsx`

**Tests to add or update**:

- 验证历史页收到 `history-updated` 事件后再次查询，并显示新记录。
- 验证事件监听器在组件卸载时解除。

## Risks & Considerations

- 事件必须只在数据库保存成功时发送，避免页面刷新后仍找不到记录。
- 自动刷新应保留当前筛选条件，并替换第一页而不是追加，避免重复记录。
- 异步注册监听器时需要处理组件已卸载的竞态，避免遗留监听器。

## Open Questions

- 无。
