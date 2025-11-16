# Changelog

## [1.2.0] - 2025-11-16

- Switch SemVer handling to the `semver` crate so prerelease/build metadata is preserved and ordering follows the spec.
- Add integration tests that exercise the CLI end-to-end, including quiet mode, tagging, identity errors, and forced tag failures.
- Introduce the `GBUMP_FORCE_TAG_FAILURE` env var to simulate tag lookup errors for testing.
- Improve error handling by centralising exits through a `fatal` helper.
- Harden tag scanning to ignore invalid UTF-8 entries and default to `0.0.0` when no valid SemVer tags exist.
- Enable optional vendored OpenSSL/musl builds via the `futures.musl` feature flag.
