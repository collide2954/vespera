# Contributions

This file gives the rules for all changes to Vespera. The full rules are in [docs/design/11-engineering.md](docs/design/11-engineering.md).

## Rules

1. Write all documentation in ASD-STE100 Simplified Technical English. This rule applies to Markdown files, commit messages, issue text and rustdoc text.
2. Write a test that fails before you write production code. Then write the minimum code that makes the test pass. Then improve the code.
3. Do not write comments in code. Use clear names and types. You can write rustdoc comments (`///` and `//!`) on the public items of library crates.
4. Keep the repository text-only. Put binary files in the asset store.
5. Use only assets with an approved license: CC0-1.0 or CC-BY-4.0, and OFL-1.1 for fonts only.
6. Make small commits. Each commit must build and pass all tests. Push each commit to `main`.
7. Use Conventional Commits for commit messages.

## Before you push

Do these steps:

1. Run the formatter: `cargo fmt --all`.
2. Run the linter: `cargo lint`. The linter uses a separate target folder, because check builds and test builds of the same dependencies can conflict.
3. Run all tests: `cargo test --workspace`.
4. Run the repository checks: `cargo xtask check-text-only`, `cargo xtask check-comments`, `cargo xtask check-docs` and `cargo xtask check-layers`.
