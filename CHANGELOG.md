# Changelog

## [1.3.0] - 2024-12-08

- Add `--tag-signed`/`-ts` flag to create GPG-signed tags via `git tag -s`, plus `--signer` to override the signing key.
- Normalize `-ts`/`-st`, block combining `--tag` and `--tag-signed`, and plumb `GBUMP_FAKE_GPG_LOG` into signed tagging for better diagnostics.
- Expand CLI integration coverage for signed tagging, signer validation, and flag conflicts; refactor fake GPG helper reuse.
- Add unit coverage to assert duplicate tag creation fails cleanly.
- Remove OpenSSL dependency by disabling default features on `git2` (no longer needed for local-only git operations).
- Update CI workflows to use `macos-latest` instead of retired `macos-13` runners.
- Remove OpenSSL installation steps from all workflow files.

## [1.2.0] - 2025-11-16

- Switch SemVer handling to the `semver` crate so prerelease/build metadata is preserved and ordering follows the spec.
- Add integration tests that exercise the CLI end-to-end, including quiet mode, tagging, identity errors, and forced tag failures.
- Introduce the `GBUMP_FORCE_TAG_FAILURE` env var to simulate tag lookup errors for testing.
- Improve error handling by centralising exits through a `fatal` helper.
- Harden tag scanning to ignore invalid UTF-8 entries and default to `0.0.0` when no valid SemVer tags exist.
- Enable optional vendored OpenSSL/musl builds via the `futures.musl` feature flag.
