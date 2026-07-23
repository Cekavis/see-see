# Data Model: See See 统一设置主窗口

No persistent entities, migrations or storage changes are required.

## Ephemeral UI State

### SettingsSection

- Values: `general`, `models`, `prompts`, `history`, `about`
- Lifetime: current `main` webview session only
- Default: `general`
- Persistence: none
- Transition: sidebar selection or onboarding navigation replaces the current value

All existing model configurations, prompt presets, history entries and application settings keep their current schemas and state transitions.
