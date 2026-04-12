# Contributing

Thanks for contributing to `jao`.

## Before You Start

- Open an issue or feature request first for larger changes.
- Keep changes focused. Avoid mixing refactors, features, and unrelated cleanups.
- Update tests and docs when behavior changes.

## Local Setup

```bash
cargo test
jao gen docs
```

For feature-combination coverage:

```bash
cargo hack build --locked --each-feature
```

## Pull Requests

- Explain the user-facing problem being solved.
- Include validation steps in the PR description.
- Update the changelog when the change matters to users.
- Keep README examples accurate when command behavior changes.

## Style

- Prefer small, direct changes over broad rewrites.
- Preserve cross-platform behavior for Unix and Windows.
- Add tests close to the code when a helper can be validated locally.

## Release Notes

If your change affects users, add a short entry under `Unreleased` in `CHANGELOG.md`.