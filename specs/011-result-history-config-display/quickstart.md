# Quickstart: Result and History Configuration Display

## Automated validation

```powershell
npm test -- src/App.test.ts src/views/Result.test.tsx src/views/History.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml --test analysis_flow
npm run lint
npm run format:check
npm run build
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected outcomes:

- Analysis snapshots expose and preserve exact configuration names.
- Retry resets the snapshot with the retry configuration names.
- Result header displays both names when available.
- History metadata displays the saved model configuration name.
- History list requests original images and uses image-first, one-column, natural-height, capped, non-cropping presentation.
- Pagination requests only 10, 20, or 50 records and previous/next use the expected cursors.

## Manual desktop validation

1. Start an analysis and verify the result window shows the active model and prompt configuration names during streaming and after completion.
2. Change active configurations, retry a retryable failure, and verify the names update to the retry configuration.
3. Open history and verify each item shows its saved model and prompt names.
4. Verify wide original screenshot text is visible above content without cropping or fixed-height blank space at 1094×768 and 540×800; verify tall images stop at the height cap.
5. Navigate next and previous pages, then select 20 and 50 items per page; confirm each change resets or advances correctly and does not accumulate earlier items.
6. Verify the result header wraps cleanly at 460×500 and 420×360 with no horizontal page overflow.
7. Verify history detail, filters, deletion, copy, resubmit, result cancel, retry, copy, open-main, and always-on-top behavior remain available.

## Release validation

```powershell
npm run tauri build
```

Install the generated Windows bundle locally and repeat the primary result/history flow before committing.
