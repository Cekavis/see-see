# US5 validation

Date: 2026-07-23

- Rust tests cover register-before-unregister shortcut replacement, conflict rollback, autostart persistence only after system success, and diagnostic-log redaction.
- Frontend tests cover onboarding readiness, permission recovery, model setup routing, autostart default off, shortcut conflict recovery, autostart updates, and log export.
- The runtime initializes the single-instance plugin, persistent global shortcut, tray/menu items, close-to-hide windows, explicit quit cleanup, system autostart reconciliation, and local log target.

Outstanding: Windows and macOS manual validation of tray/menu behavior, duplicate launch, real shortcut conflict, OS autostart, and permission dialogs remains required, so T064 stays open.
