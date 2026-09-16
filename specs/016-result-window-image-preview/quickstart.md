# Quickstart: Result Window Image Preview

## Prerequisites

- Windows or macOS desktop build of See See.
- An active model configuration capable of accepting an image.
- A prompt configuration.
- One wide screenshot and one tall screenshot available to capture.

## Automated validation

From the repository root:

```powershell
powershell.exe -Command "npm run typecheck"
powershell.exe -Command "npm test -- src/views/Result.test.tsx src/App.test.ts"
cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle
powershell.exe -Command "npm run lint"
powershell.exe -Command "npm run format:check"
```

Expected results:

- The focused frontend tests confirm the preview is rendered with the correct accessible label, result text/footer remain available, and the four-row result layout keeps the text row scrollable.
- The Rust lifecycle test confirms the result window remains 460 pixels wide by default, uses a 750-pixel default height, and has a 420×540 minimum.
- Typecheck, lint, and formatting complete without new errors.

## Manual visual validation

1. Launch See See and trigger an analysis using a wide screenshot.
2. At the default 460×750 result window, confirm the original screenshot appears in a single top preview row, scales to the window width, and is capped at the preview height without horizontal overflow.
3. Confirm streaming text, thinking disclosure, token usage, and footer actions remain usable while the model is responding.
4. Confirm the completed result remains scrollable when the answer is long and the preview remains above it.
5. Resize the result window to 420×540 and repeat the checks, including keyboard focus on footer actions.
6. Repeat with a tall screenshot and verify the image keeps its proportions while respecting the height cap.
7. Repeat in light and dark appearances when available.
8. If image retrieval is forced to fail during development, confirm the existing notification appears while result content and actions remain usable.

## Evidence to record

Record pass/fail notes (and screenshots when available) for:

- Wide screenshot at 460×750 and 420×540.
- Tall screenshot at 460×750 and 420×540.
- Streaming and completed result states.
- Light and dark appearances.
- No horizontal overflow, preserved aspect ratio, bounded preview height, reachable footer, and scrollable result text.
