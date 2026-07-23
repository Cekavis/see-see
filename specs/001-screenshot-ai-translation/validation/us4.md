# US4 validation

Date: 2026-07-23

- `cargo test --release --manifest-path src-tauri/Cargo.toml --test history_benchmark -j 1` passed; 10,000-record open, escaped substring search, and prompt filtering completed in 0.03 seconds.
- The first release build hit the 600-second command limit during dependency compilation; the cached rerun completed in 163.8 seconds and the benchmark itself passed.
- Integration tests cover stable cursor pagination, literal `%`/`_` search, prompt/status filters, original/thumbnail binary reads, corrupt-image errors, cascade deletion, clear plus `VACUUM`, and the persisted history toggle.
- Database initialization tests verify `foreign_keys=ON` and `secure_delete=ON`; history-disabled analysis does not persist entry or image rows.
- Frontend tests cover loading, empty/no-result behavior, search/filter, detail, copy, resubmit, single deletion, and full clear confirmation.
