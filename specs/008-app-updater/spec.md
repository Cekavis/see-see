# Feature Specification: App Updater

**Feature Branch**: `master`

**Created**: 2026-08-06

**Status**: Draft

**Input**: User description: "增加完整的 GitHub 发布流程，自动发布 Windows 和 macOS 安装包及说明，并增加检查更新一键安装功能"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Check for Updates (Priority: P1)

As a user, I can check for a newer See See version from the About page and immediately understand whether I am current, an update is available, or the check failed.

**Why this priority**: Users need a trustworthy update result before any installation action is offered.

**Independent Test**: Open About, trigger checks that return no update, a newer version, and a network failure, then verify the current version, status, recovery action, and available release notes.

**Acceptance Scenarios**:

1. **Given** the installed version is current, **When** the user checks for updates, **Then** the interface confirms that no newer version is available.
2. **Given** a newer signed release exists, **When** the user checks for updates, **Then** the interface shows its version, release notes, and an install action.
3. **Given** the check cannot complete, **When** the request fails, **Then** the interface explains the failure and allows the user to retry.

---

### User Story 2 - Install and Restart (Priority: P2)

As a user who found an update, I can start installation with one action, see download progress, and return to the updated application after restart.

**Why this priority**: The update flow only delivers value when users can complete installation without manually downloading a package.

**Independent Test**: Provide a valid signed update, start installation, emit download progress, complete installation, and verify that duplicate actions are disabled and the application restarts.

**Acceptance Scenarios**:

1. **Given** a valid update is available, **When** the user selects install, **Then** the application downloads and installs it without requiring another in-app confirmation.
2. **Given** installation is active, **When** progress events arrive, **Then** the interface reports progress and prevents a second check or install action.
3. **Given** installation completes, **When** the platform permits restart, **Then** See See relaunches into the new version.
4. **Given** download or installation fails, **When** the error is returned, **Then** the current installation remains usable and the user can retry.

---

### User Story 3 - Publish Update-Ready Releases (Priority: P3)

As a maintainer, I can push one annotated version tag and receive a published GitHub release containing user installers, signed updater artifacts, metadata, and generated notes for supported Windows and macOS architectures.

**Why this priority**: In-app installation depends on complete, signed, discoverable releases.

**Independent Test**: Run the release workflow for a version tag and verify that an incomplete or unsigned build remains a draft while a complete build publishes all installers and valid updater metadata.

**Acceptance Scenarios**:

1. **Given** a tag does not match the synchronized application version, **When** the workflow starts, **Then** it fails before creating a public release.
2. **Given** required signing material is unavailable, **When** the workflow starts, **Then** it fails before creating a public release.
3. **Given** every platform build succeeds, **When** all installers, signatures, and updater metadata are present, **Then** the draft is published with generated notes and marked latest.
4. **Given** any platform build or asset check fails, **When** the workflow ends, **Then** the release remains unpublished for safe retry.

### Edge Cases

- The device is offline, GitHub is unavailable, or the request times out.
- The latest release matches or is older than the installed version.
- Update metadata is malformed, incomplete, or signed by an unknown key.
- The user activates check or install repeatedly while an operation is in progress.
- Release notes are empty or contain multiple paragraphs.
- The application window closes or the component unmounts during a check.
- Windows installer shutdown occurs before an explicit relaunch call completes.
- A workflow retry encounters an existing draft with some assets already uploaded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The About page MUST display the packaged application version and provide a manual check-for-updates action.
- **FR-002**: The system MUST distinguish checking, current, update-available, installing, installed/restarting, and failed states.
- **FR-003**: An available update MUST show its version and any supplied release notes before installation.
- **FR-004**: The user MUST be able to download and install an available update with one install action.
- **FR-005**: The interface MUST report download progress when total size information is available and MUST prevent duplicate update operations.
- **FR-006**: The application MUST relaunch after a successful installation when supported by the platform updater.
- **FR-007**: Failed checks or installations MUST leave the current application usable and provide a retry path.
- **FR-008**: Every update MUST be cryptographically verified against the public key embedded in the application; verification MUST NOT be optional.
- **FR-009**: The release workflow MUST create signed updater artifacts and metadata for Windows x64, macOS Apple Silicon, and macOS Intel.
- **FR-010**: The release workflow MUST publish Windows x64 MSI and NSIS installers plus Apple Silicon and Intel DMGs with generated release notes.
- **FR-011**: A release MUST remain a draft until all required installers and updater metadata pass validation.
- **FR-012**: Version tags MUST match the synchronized application version files before any release is published.
- **FR-013**: Private signing keys, certificate files, and passwords MUST remain outside source control and be supplied only through protected local storage or repository secrets.

### User Experience and UI Requirements *(mandatory for user-facing features)*

- **UX-001**: Reuse the existing About page, button, notification, loading, error, and recovery patterns.
- **UX-002**: Keep update status adjacent to the installed version and show only decision-relevant text such as availability, notes, progress, and errors.
- **UX-003**: Disable check and install controls while an update operation is active and expose progress through accessible text.
- **UI-001**: Reuse existing settings groups, rows, typography, spacing, focus states, and responsive behavior without a new visual system.
- **UI-002**: All update actions MUST be reachable by keyboard, retain visible focus, and expose busy/disabled state to assistive technology.
- **UI-003**: Human visual review MUST cover the About page at normal desktop width and the existing compact breakpoint.

### Key Entities

- **Update Availability**: The checked release version, optional release notes, and whether it is newer than the installed version.
- **Update Operation**: The current check/download/install state, downloaded bytes, optional total bytes, and recoverable failure.
- **Published Release**: The version tag, release notes, platform installers, signed updater artifacts, and updater metadata.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Under a stable connection, users receive a current/update-available result within 10 seconds of activating the check action.
- **SC-002**: Users can move from an available-update result to installation with one additional activation.
- **SC-003**: During download, the interface updates progress without accepting duplicate check or install actions.
- **SC-004**: Invalid or unsigned update metadata is rejected before the current installation is replaced.
- **SC-005**: A successful tagged release contains two Windows installers, two macOS DMGs, signed updater artifacts for all three target architectures, one updater metadata file, and generated release notes.
- **SC-006**: No release becomes public when a required platform build, signature, installer, or metadata entry is missing.

## Assumptions

- Updates are user-initiated; automatic background installation is out of scope.
- GitHub Releases is the distribution endpoint, and the repository remains reachable over HTTPS.
- Windows x64, macOS Apple Silicon, and macOS Intel remain the supported release targets.
- The existing stable macOS signing identity is reused in CI, while updater artifacts use a separate Tauri signing key.
- Release notes are generated from GitHub history and may be supplemented by concise installer guidance.
