# US5 validation

Date: 2026-07-23

- Rust tests cover register-before-unregister shortcut replacement, conflict rollback, autostart persistence only after system success, and diagnostic-log redaction.
- Frontend tests cover onboarding readiness, permission recovery, model setup routing, autostart default off, shortcut conflict recovery, autostart updates, and log export.
- The runtime initializes the single-instance plugin, persistent global shortcut, tray/menu items, close-to-hide windows, explicit quit cleanup, system autostart reconciliation, and local log target.

Outstanding: Windows and macOS manual validation of tray/menu behavior, duplicate launch, real shortcut conflict, OS autostart, and permission dialogs remains required, so T064 stays open.

## macOS Login Item update — 2026-07-26

- Host: macOS 27.0 arm64.
- The signed and installed 0.3.3 app registered its main application through `SMAppService.mainAppService`; the command completed only after `SMAppService.status` returned enabled.
- The app switch and persisted `autostart` value changed to on after registration and returned to off after successful unregistration.
- No `~/Library/LaunchAgents/See See.plist` was created. The test restored the original off state.
- System Settings navigated to the Login Items panel, but the automation accessibility bridge closed while reading the item rows. A human visual confirmation of the See See row and a logout/login launch check remain outstanding, so T064 stays open.
