# Bug Assessment: Result copy button is clipped

- **Slug**: result-copy-button-clipped
- **Created**: 2026-07-26
- **Source**: pasted text and app screenshot
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

The user reports: “这里最下面的按钮只露出来一部分，请你修复”. In the supplied See See result-window screenshot, the “复制全文” button is partially below the bottom edge while a long recognition result is displayed.

## Symptom

Long recognition output expands the text area beyond the result window, pushing the footer below the viewport so only part of the copy button remains visible. The text area should scroll within the available space while the footer remains fully visible.

## Reproduction

1. Complete a recognition run that produces output taller than the result window.
2. View the result in the 460 × 500 result window.
3. Observe that the text region expands with its contents and clips the bottom copy button.

## Suspected Code Paths

- `src/styles.css:1161` — `.result-view` defines four grid rows for three direct children, placing the text area in an unconstrained `auto` row.
- `src/views/Result.tsx:46` — the result view has exactly three direct grid children: header, text, and footer.
- `src/views/Result.test.tsx` — existing behavior tests do not assert the layout contract that keeps the footer visible.

## Root Cause Hypothesis

Confidence: high. `.result-view` declares `grid-template-rows: auto auto minmax(0, 1fr) auto`, but renders only three direct children. The text area is therefore assigned to the second `auto` row and grows to its content height; the footer occupies the flexible third row instead of the final fixed row. Since the container has `height: 100vh` and `overflow: hidden`, the oversized content and footer are clipped at the window edge.

## Proposed Remediation

**Preferred**: Change the result grid to three rows—`auto minmax(0, 1fr) auto`—so the header and footer keep their intrinsic heights and the text area consumes the remaining height with its existing internal scrollbar. Add a focused regression test that reads the stylesheet and pins the three-row layout contract.

**Files likely to change**:

- `src/styles.css`
- `src/views/Result.test.tsx`

**Tests to add or update**:

- Verify `.result-view` uses `auto minmax(0, 1fr) auto` and no longer has an extra `auto` row before the flexible row.

## Risks & Considerations

- The fix affects only the result-window grid; the history detail reuses the text class but not the result grid.
- jsdom does not perform layout, so the automated regression must pin the CSS structure; the packaged app should also be checked visually.

## Open Questions

- None.
