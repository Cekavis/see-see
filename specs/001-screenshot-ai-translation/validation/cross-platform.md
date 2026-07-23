# Cross-platform validation

Date: 2026-07-23

## Completed on Windows

- Rust coordinate tests passed for negative origins, reverse drag, zero-size cancellation, and cross-monitor pixel composition.
- A Windows x64 Tauri development build compiled and launched `see-see.exe`.
- Windows x64 release executable, MSI, and NSIS installer were generated successfully.

## Not yet executed

The quickstart physical-device matrix has not been completed. Required remaining runs are:

- Windows single display at 100%, high-DPI display, mixed-DPI dual display, left-side negative coordinates, and real cross-display dragging.
- macOS 14+ single/high-DPI/dual display, screen-recording permission denial/grant, menu-bar behavior, and universal release build.
- Screenshot evidence confirming overlays are absent and captured pixels match each visible selection.

T027 and T068 remain open. Automated coordinate tests are not a substitute for OS compositor, permission, and display-scaling validation.
