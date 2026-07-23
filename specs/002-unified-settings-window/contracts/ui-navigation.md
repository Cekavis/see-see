# UI and Desktop Lifecycle Contract

## Main window

- The `main` window is the only management window.
- Its default section is `general`.
- Section navigation never creates a Tauri window.
- Closing the window hides it; reopening or launching a second instance shows and focuses the same window.

## Screenshot entry points

- The main window contains no control named “开始截图”.
- The configured global shortcut remains active.
- The tray/menu-bar item `capture` invokes the existing capture action.

## Tray/menu-bar commands

| ID | Label | Result |
|----|-------|--------|
| `capture` | 开始截图 | Invoke the existing capture action asynchronously |
| `show` | 打开 See See | Show and focus `main` |
| `quit` | 退出 | Exit the application |

## Removed internal contract

- Remove the frontend `openView` wrapper.
- Remove the Rust `open_view` command and generated handler entry.
- Remove handling for `settings`, `prompts` and `history` window labels.
