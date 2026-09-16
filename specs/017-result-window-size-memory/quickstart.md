# Quickstart Validation: Result Window Size Memory

## Prerequisites

- A configured model and prompt that can start a screenshot analysis.
- At least one saved history item with an image, or a completed analysis that is saved to history.

## Automated checks

1. Run focused Rust storage and result-window lifecycle tests.
2. Run focused frontend Result and History tests.
3. Run formatter, lint, typecheck, frontend tests, and the relevant Rust tests.

## Manual result-window validation

1. Start an analysis with no saved window preference.
2. Confirm the result window opens at 460×540 and retains its existing 420×540 minimum.
3. Resize it to a clearly different valid size, then start another analysis while the app remains open. Confirm the newer result uses that size.
4. Close a result, quit the app through its tray/menu action, reopen the app, and start another analysis. Confirm the same size is restored.
5. Resize to the minimum and to a larger size; confirm result scrolling, copy, retry/cancel where applicable, open-main-window, and always-on-top remain reachable.

## Manual preview validation

1. Use both a tall and a wide screenshot.
2. Confirm the result preview is no taller than 60 pixels, preserves its proportions, and has no horizontal overflow.
3. Open the history list and the history detail for the same images. Confirm both image presentations also remain no taller than 60 pixels, preserve proportions, and do not horizontally overflow.
4. Review the result at 460×540, at 420×540, and at the remembered larger size in the available light/dark appearances.
