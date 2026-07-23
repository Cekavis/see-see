# Research: See See 统一设置主窗口

## Decision 1: One React-owned management window

**Decision**: Use a local section state in the `main` webview and remove the internal `open_view` command.

**Rationale**: All management content already exists in React. Local navigation deletes dynamic window creation and avoids adding a router or shared state library.

**Alternatives considered**: Keeping independent windows would preserve the current fragmented experience; emitting cross-window navigation events would retain unnecessary infrastructure.

## Decision 2: Native platform primitives first

**Decision**: Use native buttons, inputs, checkboxes and `<dialog>`, styled with CSS tokens and inline SVG icons.

**Rationale**: This preserves accessibility and avoids new dependencies while supporting the requested visual language.

**Alternatives considered**: A component library or icon package would add ownership and bundle cost without solving a missing capability.

## Decision 3: Reuse the existing capture action

**Decision**: The tray capture item invokes `begin_capture_action`, the same path used by the global shortcut and IPC command.

**Rationale**: A single capture path preserves concurrency checks, validation and error behavior.

**Alternatives considered**: Duplicating capture setup in the tray handler would create a second failure path.

## Decision 4: CSS theme, no window translucency

**Decision**: Follow `prefers-color-scheme` with opaque surfaces, a static low-contrast background gradient and no platform blur APIs.

**Rationale**: It matches the supplied Windows reference while keeping Windows and macOS behavior consistent and testable.

**Alternatives considered**: Mica, Acrylic, Vibrancy, transparent windows and custom titlebars require platform-specific behavior outside the requested scope.
