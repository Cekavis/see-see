# Research: Tray Click Settings

## Decision 1: Use the existing tray builder behavior switch

- **Decision**: Disable menu display on primary click with the installed Tauri tray builder option.
- **Rationale**: Tauri already exposes this behavior directly, so no custom menu routing or platform-specific code is needed.
- **Alternatives considered**: Removing the menu and displaying it manually was rejected because it duplicates native tray behavior and adds platform risk.

## Decision 2: Open settings on primary-button release

- **Decision**: Treat a left-button `Up` event as the completed click that opens the settings window.
- **Rationale**: The installed tray implementation emits both press and release events on Windows and macOS; handling release avoids opening twice for one click.
- **Alternatives considered**: Handling every left-button event was rejected because one physical click would trigger twice. Handling double-click was rejected because it is a separate Windows-only event.

## Decision 3: Reuse the existing window function

- **Decision**: Call the existing `show_main_window` helper.
- **Rationale**: It already unminimizes, shows, and focuses the single configured settings window.
- **Alternatives considered**: Creating a new window or duplicating the show/focus calls was rejected because both increase code and risk inconsistent behavior.
