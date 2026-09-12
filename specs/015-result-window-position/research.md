# Research: Remember Result Window Position

## Decision 1: Keep the position in application runtime state

- **Decision**: Store one optional latest result-window position on `RuntimeState`.
- **Rationale**: `RuntimeState` is already shared by the Tauri application and is protected by the existing mutex. The requested behavior concerns windows created after a move, and the specification scopes it to the current application session. This avoids a schema change and avoids disk I/O for high-frequency move events.
- **Alternatives considered**: Persisting coordinates in `app_settings` was rejected because cross-restart restoration was not requested and would add migrations, validation, and multi-display lifecycle concerns.

## Decision 2: Use physical screen coordinates

- **Decision**: Capture the coordinates supplied by Tauri's native moved-window event and apply them with the window's physical-position setter.
- **Rationale**: The installed Tauri runtime exposes `WindowEvent::Moved(PhysicalPosition<i32>)`, and `WebviewWindow::set_position` accepts a physical position. This preserves the top-left screen location across display scale factors and avoids treating physical coordinates as logical builder coordinates.
- **Alternatives considered**: Passing coordinates to `WebviewWindowBuilder::position` was rejected because that builder API takes logical pixels, which would require a scale-factor conversion and could introduce rounding or monitor mismatches.

## Decision 3: Filter movement events by result-window label

- **Decision**: Update the shared position only when `result_run_id(window.label())` recognizes the window as a result window.
- **Rationale**: The application also creates capture overlays and a main management window. Their movements must never change the default placement for result windows.
- **Alternatives considered**: Recording every `Moved` event was rejected because moving the capture overlay or main window would create surprising result placement changes.

## Decision 4: Preserve the existing centered fallback and presentation order

- **Decision**: With no remembered position, keep `center()`; with a remembered position, set it before `show()` and `set_focus()`.
- **Rationale**: This preserves current first-use behavior while ensuring the new position is applied while the window is still hidden. Existing macOS policy setup remains on the main thread, and existing focus/always-on-top behavior is untouched.
- **Alternatives considered**: Moving the window after showing it was rejected because it can cause visible placement jumps and can emit an extra movement event during presentation.

## Decision 5: Treat the latest move as authoritative

- **Decision**: Every result-window movement replaces the previous value, and closing a result window does not clear it.
- **Rationale**: This matches the user's wording and supports concurrent result windows: whichever result window was moved most recently determines where the next new result window opens.
- **Alternatives considered**: Keeping a position per analysis run or clearing the value when its owner closes was rejected because the requested preference is shared across result windows rather than tied to one analysis.
