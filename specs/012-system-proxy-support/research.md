# Research: System Proxy Support

## Decision 1: Use reqwest's built-in system proxy discovery

- **Decision**: Enable reqwest's `system-proxy` feature for the model client's locked 0.12 line.
- **Rationale**: The existing `Client::builder()` already enables automatic proxy matching by default. With the feature enabled, reqwest checks environment variables first and then supported Windows/macOS proxy configuration.
- **Alternatives considered**: Reading registry/SystemConfiguration values in application code was rejected because it duplicates library behavior and requires custom proxy parsing and bypass handling.

## Decision 2: Enable the updater's reqwest 0.13 feature through Cargo feature unification

- **Decision**: Add a direct aliased reqwest 0.13 dependency with only `system-proxy` enabled so Cargo unifies that feature with tauri-plugin-updater's reqwest 0.13 instance.
- **Rationale**: tauri-plugin-updater constructs its own reqwest client and does not expose `system-proxy` as a plugin feature. Cargo feature unification enables the supported behavior without forking or replacing the updater.
- **Alternatives considered**: Upgrading the application's model client from reqwest 0.12 to 0.13 was rejected because it would unify native-tls and rustls features and unnecessarily change the model request dependency version. Manually setting a proxy on the updater was rejected because the application would still need to discover and parse operating-system configuration.

## Decision 3: Guard the behavior at configuration level

- **Decision**: Extend `scripts/verify-release-config.mjs` to assert that both reqwest dependency paths enable `system-proxy`, and validate resolved features with `cargo tree -e features`.
- **Rationale**: The application adds no proxy-selection logic to unit test. A configuration regression assertion fails if the enabling features are removed, while Cargo's resolved feature tree proves that both versions receive the feature.
- **Alternatives considered**: Automated mutation of Windows/macOS global proxy configuration was rejected as unsafe, platform-specific, and unsuitable for routine tests.

## Decision 4: Publish version 0.11.0

- **Decision**: Increment the synchronized application version from 0.10.2 to 0.11.0.
- **Rationale**: System proxy support is a backward-compatible user-facing feature.
- **Alternatives considered**: A patch release was rejected because the repository's version policy assigns minor releases to compatible features.
