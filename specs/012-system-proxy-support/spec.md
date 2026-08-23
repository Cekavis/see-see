# Feature Specification: System Proxy Support

**Feature Branch**: `master`

**Created**: 2026-08-23

**Status**: Published

**Input**: User description: "改成接受系统代理，然后发布release"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use the operating system proxy for model requests (Priority: P1)

As a user whose network requires the proxy configured in Windows or macOS, I can list models, test a model connection, and run screenshot analysis without separately configuring proxy environment variables.

**Why this priority**: Model access is the application's primary online workflow and is currently blocked for users who rely only on operating-system proxy settings.

**Independent Test**: Configure a reachable operating-system proxy, leave proxy environment variables unset, and verify that model listing, connection testing, and analysis requests reach the configured endpoint through the proxy.

**Acceptance Scenarios**:

1. **Given** a valid operating-system HTTP or HTTPS proxy and no proxy environment variables, **When** the user lists or tests models, **Then** the request uses the operating-system proxy and returns the provider response.
2. **Given** a valid operating-system proxy and no proxy environment variables, **When** the user analyzes a screenshot, **Then** the streaming model request uses the operating-system proxy and completes normally.
3. **Given** no proxy is configured, **When** the user performs a model request, **Then** the request continues to connect directly as before.

---

### User Story 2 - Use the operating system proxy for updates (Priority: P2)

As a user whose network requires the proxy configured in Windows or macOS, I can check for and download See See updates through that proxy.

**Why this priority**: Users behind a proxy must be able to receive the release that contains this capability and future releases without manual environment configuration.

**Independent Test**: Configure a reachable operating-system proxy, leave proxy environment variables unset, and verify that the updater can retrieve release metadata and download an update asset.

**Acceptance Scenarios**:

1. **Given** a valid operating-system proxy and no proxy environment variables, **When** See See checks for updates, **Then** the updater retrieves release metadata through the proxy.
2. **Given** an available update and a valid operating-system proxy, **When** the user downloads it, **Then** the update package is downloaded through the proxy and retains existing signature verification.

### Edge Cases

- Proxy exclusion rules configured by the environment or operating system must continue to bypass the proxy for matching destinations.
- An unavailable or invalid proxy must surface through the existing network error and recovery behavior rather than silently falling back to a direct connection.
- Explicit proxy environment variables must remain supported and take precedence according to the networking library's established behavior.
- Enabling proxy discovery must not permit redirects that the model client currently rejects or weaken updater signature verification.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: See See MUST discover proxy settings from supported Windows and macOS operating-system configuration for model network requests.
- **FR-002**: System proxy discovery MUST apply to model listing, connection testing, and screenshot analysis because they share the same network client.
- **FR-003**: See See MUST discover supported operating-system proxy settings for update metadata checks and update downloads.
- **FR-004**: Existing `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY` environment-variable behavior MUST remain available.
- **FR-005**: Direct connections MUST continue to work when no applicable proxy is configured.
- **FR-006**: Existing redirect restrictions, timeouts, TLS validation, credential handling, and updater signature verification MUST remain unchanged.
- **FR-007**: The release MUST use the next compatible-feature version and publish through the repository's existing signed release workflow.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On both Windows and macOS, all three model workflows succeed through a valid operating-system proxy with proxy environment variables unset.
- **SC-002**: Update metadata retrieval and package download succeed through a valid operating-system proxy with proxy environment variables unset.
- **SC-003**: Existing direct-connect and environment-variable proxy workflows pass without regressions.
- **SC-004**: The feature is published as a signed release containing all required Windows and macOS installer and updater artifacts.

## Assumptions

- The operating system proxy uses a scheme supported by the existing networking libraries.
- Proxy configuration is established before See See creates its network clients or performs an update request.
- Adding an application-specific proxy settings interface is out of scope; See See follows the operating system and existing proxy environment variables.
- Linux-specific desktop proxy discovery is out of scope because the supported release targets are Windows and macOS.
