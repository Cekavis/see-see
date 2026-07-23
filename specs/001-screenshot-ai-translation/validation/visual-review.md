# Visual and accessibility review

Date: 2026-07-23

## Browser evidence completed

The local bundled frontend was inspected in the in-app browser at the required representative sizes. Evidence:

- `screenshots/main-1024x720.png`
- `screenshots/main-1440x900.png`
- `screenshots/main-360x240.png`
- `screenshots/main-1024x720-200-percent.png`

Observed DOM/layout results:

- 1024×720: no horizontal or vertical overflow; loading, error, retry, capture, model, prompt, and history controls are visible.
- 1440×900: no overflow; hierarchy and spacing remain consistent.
- 360×240: no horizontal overflow; vertical scrolling is required and available, with primary controls remaining reachable below the fold.
- A Chrome page-scale probe at 200% kept all five buttons present and visible, but this is not equivalent to Windows/macOS system text scaling.
- Focus styles and keyboard-focusable native buttons/inputs are defined in shared CSS and covered by component tests, but a complete keyboard traversal was not manually recorded.

## Outstanding review

- Native Windows screenshots for onboarding, settings, prompts, result, history empty/list/detail, and provider error states.
- Native macOS review at all required sizes and states.
- Real 200% operating-system text scaling, contrast measurement, full keyboard focus order, reduced-motion behavior, and platform convention review.

T069 remains open because the constitution requires human review of all named views on both platforms.
