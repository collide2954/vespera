# 11. Engineering rules

## 11.1 Toolchain

- The code uses Rust edition 2024. The file `rust-toolchain.toml` pins one stable Rust version.
- The project starts with Bevy 0.20. It moves to each new Bevy release when the main ecosystem crates work with that release. The main ecosystem crates are `bevy_replicon`, `avian3d`, `bevy_yarnspinner`, `bevy_enhanced_input` and `bevy_kira_audio`.
- Development builds use dynamic links for Bevy, the LLD linker and optimized dependencies. These settings make builds faster.
- On Windows, `cargo xtask fetch-dxc` downloads the official DXC release of Microsoft and verifies its SHA-256 hash. Then it puts `dxcompiler.dll` and its licenses next to the binaries in `target`. Run it again after `cargo clean`.
- DXC uses the MIT license and the University of Illinois/NCSA license. Each release package for Windows must contain `dxcompiler.dll` and these licenses.
- In development builds, the debug information of dependencies contains only line tables. Full debug information made the linker use all memory when many test binaries linked at the same time.

## 11.2 Test-driven development

All work uses test-driven development (TDD), from the top to the bottom.

1. Write an acceptance test for the feature. The test describes the behavior that a player sees.
2. Run the test. Make sure that it fails.
3. Write unit tests for the parts below, one at a time. Make sure that each test fails first.
4. Write the minimum code that makes each test pass.
5. Improve the code. Keep all tests green.
6. Make a commit.

The name of a test tells the behavior, for example `player_receives_oak_logs_after_cutting_an_oak_tree`.

## 11.3 Test levels

| Level | Tool | Example |
|---|---|---|
| Acceptance | `vespera_testkit`: a headless server and clients in one process, with a simulated network | A player cuts a tree and receives logs. |
| Integration | cargo-nextest | The SQLite store and the PostgreSQL store save and load a character. |
| System | A Bevy `App` with minimal plugins | The gathering system gives XP. |
| Unit | cargo-nextest | The XP curve, the order book and two-bone IK. |
| Property | proptest | Trades do not make or destroy coins. |
| Shader | naga validation | All shaders compile. |
| Visual | A software GPU: lavapipe on Linux, WARP on Windows | The valley at dusk agrees with its golden screenshot. |
| Performance | criterion and `cargo xtask bench` | The rules tick for 1,000 players. |
| Mutation | cargo-mutants | Each surviving mutant causes a new test. |

## 11.4 Code rules

- Do not write `//` or `/* */` comments. Use clear names, small functions and strong types.
- You can write rustdoc comments (`///` and `//!`) on the public items of library crates. Write them in ASD-STE100.
- If a normal comment is absolutely necessary, add it to `comment-allowlist.toml` with a reason. A reviewer must accept it.
- Do not write `unsafe` code. The workspace forbids it.
- Do not use `unwrap` or `expect` outside tests. Return a `Result` with context.
- The server must not panic because of client input.
- Do not use global mutable state.
- Each feature is a Bevy plugin with its own system sets.
- Obey the layer rules for crates (see 8.3).
- Each new dependency must have a reason in the commit message.

## 11.5 Lints

The workspace `Cargo.toml` sets the lints for all crates:

- `unsafe_code` is forbidden.
- `clippy::all` and `clippy::pedantic` are errors in CI.
- `clippy::unwrap_used` and `clippy::expect_used` are errors outside tests.
- Some lints do not agree with the project rules or with Bevy patterns. These lints are exceptions. The configuration files contain no comments. Thus the table below gives the reason for each exception.

| Lint | Reason |
|---|---|
| `clippy::missing_errors_doc` | Rustdoc comments are optional (D28). This lint makes a rustdoc section necessary for each public function that returns a `Result`. |
| `clippy::missing_panics_doc` | Rustdoc comments are optional (D28). This lint makes a rustdoc section necessary for each public function that can panic. |
| `clippy::needless_pass_by_value` | Bevy systems receive their parameters, for example `Res` and `Query`, by value. |
| `clippy::type_complexity` | Bevy queries with filters are long types by nature. A type alias for each query makes the systems harder to read. |

## 11.6 Continuous integration

GitHub Actions runs these jobs for each push to `main`:

| Job | Platforms | The job fails when |
|---|---|---|
| Format | Linux | rustfmt finds a change. |
| Lint | Linux | clippy finds a warning. |
| Test | Windows, Linux and macOS | A test fails. |
| Database | Linux, with PostgreSQL | A persistence test fails on PostgreSQL. |
| Coverage | Linux | The line coverage is less than 90%. The job measures all crates except the presentation crates. |
| Dependencies | Linux | cargo-deny finds a license that is not in `deny.toml`, a security advisory or a bad source. `deny.toml` gives the reason for each ignored advisory. |
| Text-only | Linux | The repository contains a binary file. |
| Comments | Linux | A `//` or `/* */` comment is not in the allowlist, or an allowlist entry has no reason or matches no comment. |
| Layers | Linux | A crate has a forbidden dependency. |
| Spelling | Linux | typos finds a spelling error. |
| Documents | Linux | A Markdown file breaks a basic ASD-STE100 rule. |
| Assets | Linux | An asset archive has a bad hash, or `reuse lint` fails. |
| Visual | Linux and Windows | A screenshot is different from its golden screenshot by more than the tolerance. |
| Mutation | Linux, one time each week | The job does not fail. It reports each surviving mutant. |

The presentation crates are `vespera`, `vespera_render`, `vespera_ui` and `vespera_audio`. Visual tests check these crates (see 11.3).

Each job starts in the milestone that adds its subject. For example, the Database job starts with the first PostgreSQL store in M2. The Text-only, Comments, Layers and Documents checks run as steps of one job with the name Repository checks.

The Documents job does only basic checks: the sentence length and a list of words that ASD-STE100 does not approve. The official ASD-STE100 dictionary is not part of the repository.

## 11.7 Commits

- Work on `main` (trunk-based development). Do not keep long-lived branches.
- Make small commits. Each commit must build and pass all tests.
- Push after each commit.
- If CI fails on `main`, fix it before all other work.
- Use Conventional Commits: `feat`, `fix`, `test`, `refactor`, `perf`, `docs`, `build`, `ci` and `chore`.
- Write the subject as an instruction, in ASD-STE100, with 72 characters or less.

## 11.8 Documentation

- Write all documentation in ASD-STE100: Markdown files, commit messages, issue text, rustdoc and the changelog.
- Keep sentences short: 20 words or less for instructions, and 25 words or less for descriptions.
- Write each paragraph, each list item and each table row on one line. The Documents check reads each line separately.
- Use the active voice and simple tenses.
- Use one word for one meaning.
- Use technical names for game and software concepts, for example "shard", "chunk" and "order book".
- When a design decision changes, change the design chapter and the decision register in the same commit.
