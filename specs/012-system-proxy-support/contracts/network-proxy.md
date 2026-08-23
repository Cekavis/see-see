# Network Proxy Contract

## Model requests

- Model listing, connection testing, and analysis streaming continue to share the existing application HTTP client.
- Proxy selection checks explicit proxy environment variables and supported operating-system proxy configuration.
- Matching bypass rules connect directly.
- If no proxy matches, the request connects directly.
- Existing redirect, TLS, timeout, authentication, and error mapping behavior is unchanged.

## Updater requests

- Update metadata checks and package downloads use the updater's existing HTTP client construction.
- Proxy selection checks explicit proxy environment variables and supported operating-system proxy configuration.
- Existing updater endpoint validation, TLS validation, package signature verification, and install behavior is unchanged.

## Failure behavior

- An unreachable selected proxy produces the existing network failure behavior.
- The application does not silently bypass an explicitly selected but unavailable proxy.
