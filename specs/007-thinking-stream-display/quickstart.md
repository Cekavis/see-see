# Quickstart: Thinking Stream Display

## Automated validation

```powershell
cargo test --manifest-path src-tauri/Cargo.toml provider_contracts
cargo test --manifest-path src-tauri/Cargo.toml analysis_flow
cargo test --manifest-path src-tauri/Cargo.toml history_integration
npm test -- --run src/App.test.ts src/views/Result.test.tsx
npm run lint
npm run format:check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm test
npm run tauri build
```

## Provider fixtures

Verify automated coverage for:

1. OpenAI-compatible `reasoning_content` followed by `content`.
2. OpenAI-compatible `reasoning_details` text.
3. Anthropic `thinking_delta` followed by `text_delta`.
4. Gemini parts with `thought: true` and ordinary text.
5. `<think>` and `</think>` split across multiple chunks.
6. Ordinary answer-only text beginning with no thinking tag.

## Windows visual review

1. Install and launch the generated Windows bundle.
2. Use a reasoning-capable image model that exposes thinking.
3. During thinking-only streaming, confirm the "思考过程" disclosure is open and updates visibly.
4. When the first answer text appears, confirm the disclosure closes without interrupting answer streaming.
5. Expand and collapse the disclosure with mouse and keyboard.
6. Confirm "复制全文" copies only the final answer.
7. Open the saved history detail, confirm thinking is present and collapsed, and confirm "复制结果" still copies only the answer.
8. Repeat at the normal result-window size and at a compact size no larger than 420 px wide or 300 px high; confirm footer controls remain visible and content scrolls.
9. Use an answer-only model and confirm no empty thinking disclosure appears in either live result or history detail.

## Platform gap

Repeat the visual interaction review on macOS before a cross-platform release. If macOS is unavailable in this session, record the gap rather than claiming verification.

## Verification record — 2026-08-05

- Rust tests passed, including provider, analysis, database migration, and history persistence coverage.
- Frontend formatting, lint, build, and all 47 Vitest tests passed.
- Windows MSI and NSIS bundles built successfully; the NSIS bundle installed successfully and the installed executable reports version 0.6.0.
- User review of the installed app exposed a nested-scroll overflow. After correction, visual review at 520×360 and compact 400×290 confirmed the outer content region clips overflow and the answer panel owns scrolling; no text crosses the answer border.
- Provider fixtures verify that line breaks arriving after a split `</think>` tag are removed before the final answer.
- No paid provider request was made during the final corrective verification; live stream transitions are covered by automated provider and UI tests.
- macOS was unavailable, so native macOS visual and installation verification remains required before cross-platform release.
