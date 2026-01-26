# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/)
and this project adheres to [Semantic Versioning](https://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Features

Commands
- `resolve-features`
- (`read-manifest`) Indicate which manifest is the workspace root

### Fixes

- (`read-manifest`) Report the correct manifest for the workspace
- (`lock-dependences`) Don't require a lockfile version
- (`read-lockfile`) Always report lockfile version

## [0.0.3] - 2025-08-13

### Features

Commands
- `locate-manifest`
- `read-manifest`
- `read-lockfile`
- `write-lockfile`
- `lock-dependencies`

## [0.0.2] - 2025-02-28

<!-- next-url -->
[Unreleased]: https://github.com/crate-ci/cargo-plumbing/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/crate-ci/cargo-plumbing/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/rust-cli/argfile/compare/e7b7555...v0.0.2
