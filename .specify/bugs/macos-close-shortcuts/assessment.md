# Bug Assessment: macOS window close shortcuts

- **Slug**: macos-close-shortcuts (auto-generated)
- **Created**: 2026-09-18
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report

> 之前我增加了通过esc关闭结果窗口、通过ctrl+w关闭结果和应用窗口的功能，Windows上可以用了。但我现在在macOS上esc和command+w好像没用。如果是没有实现的话，请你加上

## Symptom

Escape should close a focused result window. Command+W should close a focused result window or hide the main window on macOS, preserving the background tray process.

## Reproduction

1. Open the installed macOS app (currently 0.16.0).
2. Focus the main window and press Command+W.
3. Focus a result window and press Escape or Command+W.
4. The user reports these shortcuts do not work; Windows Ctrl+W and Escape work.

## Suspected Code Paths

- `src/App.tsx:shouldCloseWindowOnKeydown` already recognizes Escape, Ctrl+W, and Command+W, but depends on DOM key events.
- `src-tauri/src/windowing.rs:install_native_close_shortcuts` registers a native WebView2 handler only on Windows; the other-platform implementation does nothing.
- `src-tauri/src/lib.rs:run` installs the main-window shortcut and handles close requests: hide main, cancel/remove only the closing result's analysis.
- `src-tauri/src/commands.rs:create_result_window` installs the same native shortcut helper on each result.

## Root Cause Hypothesis

High confidence: macOS has no native close-shortcut handler. The DOM fallback cannot handle keys consumed by AppKit or WKWebView before DOM delivery. The native gap is confirmed in source; desktop reproduction is still to be exercised during validation.

## Proposed Remediation

**Preferred**: Install one application-local AppKit key-down monitor during macOS setup, before events reach the native responder chain. Match the event's native window to an existing Tauri main/result window and invoke its existing `close()` path. Consume only recognized close shortcuts. Keep the monitor for the application's lifetime, rather than installing a monitor for every transient result window. Use existing objc2/block2 dependencies.

**Files likely to change**:

- `src-tauri/src/windowing.rs`: native monitor, shortcut predicate, focused regression tests.
- `src-tauri/src/lib.rs`: macOS setup registration.
- `src/App.test.ts`: Command+W and modifier/window-scope regression coverage.
- The five synchronized version files: patch version 0.16.1; enable required existing AppKit features in Cargo.toml.

**Tests to add or update**:

- Escape closes only result windows; Command+W closes main/results.
- Modified Escape, unrelated windows, and Option/Shift+Command+W do not trigger close.
- Caps Lock does not disable shortcuts; the existing Ctrl+W fallback remains available.
- Native desktop smoke verification, plus frontend/Rust checks and signed local installation.

## Risks & Considerations

- Scope monitoring to this app and the event's own window so shortcuts do not affect another application or another result.
- Preserve close-request lifecycle handling and consume handled events to prevent duplicate dispatch.
- Apple documents local monitors as running on the main thread and retained by AppKit: [Monitoring Events](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/EventOverview/MonitoringEvents/MonitoringEvents.html).
- Windows desktop verification requires a Windows host; report its availability accurately.

## Open Questions

None blocking implementation.
