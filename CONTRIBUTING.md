# Contributing to Cobogo

Thanks for your interest in improving Cobogo! This is an early-stage Rust port of [Clay](https://github.com/nicbarker/clay), and contributions of all sizes are welcome — from fixing a typo to pinning down exact layout geometry in tests.

Please be respectful and constructive; this project follows its [Code of Conduct](CODE_OF_CONDUCT.md).

## Prerequisites

- **Rust 1.85 or newer** with the **2024 edition** (the crate declares `rust-version = "1.85"`). Install via [rustup](https://rustup.rs/):

  ```sh
  rustup toolchain install stable
  rustup component add rustfmt clippy
  ```

There are no other system dependencies — no build scripts, no code generation, no C toolchain.

## Getting started

Fork the repository, then clone your fork:

```sh
git clone https://github.com/<your-username>/cobogo.git
cd cobogo
cargo build --workspace
cargo test  --workspace
```

If everything builds and the tests pass, you're ready to make changes.

## Repository layout

Cobogo is a single Cargo workspace with three crates:

| Path | Crate | Published? | What it is |
|------|-------|-----------|------------|
| `.` (root `src/`) | `cobogo` | yes | The zero-dependency core layout engine |
| `renderers/ratatui` | `cobogo-renderer-ratatui` | yes | Terminal renderer built on ratatui |
| `examples/tui-app` | `tui-app` | no | Interactive example app |

The layout algorithm itself lives in `src/layout_calc.rs`; the public entry point and element construction API are in `src/context.rs`.

## Development workflow

Before opening a pull request, make sure the following all pass locally — these are the same checks a reviewer will run:

```sh
cargo fmt --all                        # format (run with --check in CI mode)
cargo clippy --workspace --all-targets # lint
cargo test  --workspace                # unit + integration + doctests
cargo doc   --no-deps                  # docs build cleanly
```

To try changes against the interactive example (requires an interactive terminal):

```sh
cargo run -p tui-app
```

### A note on tests

The current tests are **smoke-level**: they confirm the engine runs and produces output, but they do **not** assert exact computed coordinates or sizes. If your change touches `layout_calc.rs` or sizing behavior, adding a test that pins specific bounding boxes is one of the most valuable contributions you can make. Correctness fixes should come with a regression test wherever practical.

## Pull request process

1. **Fork** the repository and create a topic branch from `main`:

   ```sh
   git checkout -b fix/text-wrap-off-by-one
   ```

2. Make your change. Keep it focused — one logical change per PR is easier to review.

3. Run the full check list above (`fmt`, `clippy`, `test`, `doc`).

4. Commit using **[Conventional Commits](https://www.conventionalcommits.org/)** (see the table below). Commit messages drive automated versioning via [release-plz](https://release-plz.dev/), so the type matters.

5. Push to your fork and open a pull request against `main`, filling in the PR template. Describe *what* changed and *why*, and note whether it affects published crate behavior.

6. A maintainer will review. Please keep the discussion on the PR so the context stays with the code.

### Conventional commit types

| Type | Use for | Version effect |
|------|---------|----------------|
| `feat` | A new feature or public API addition | minor bump |
| `fix` | A bug fix | patch bump |
| `docs` | Documentation only | none |
| `refactor` | Code change that neither fixes a bug nor adds a feature | none |
| `perf` | A performance improvement | patch bump |
| `test` | Adding or fixing tests | none |
| `build` | Build system, dependencies, or Cargo metadata | none |
| `ci` | CI configuration | none |
| `chore` | Maintenance that doesn't touch `src/` | none |
| `style` | Formatting, whitespace (no behavior change) | none |

A breaking change is marked with a `!` after the type (e.g. `feat!:`) or a `BREAKING CHANGE:` footer, and triggers a major bump.

Optionally scope a commit to a crate or module, e.g. `feat(renderer): support underline text` or `fix(layout_calc): correct grow distribution`.

## Reporting bugs and requesting features

Use the issue templates: [bug report](.github/ISSUE_TEMPLATE/bug_report.yml) or [feature request](.github/ISSUE_TEMPLATE/feature_request.yml). For a layout bug, a minimal `ElementDeclaration` tree that reproduces the wrong geometry is enormously helpful.

For security issues, please follow [SECURITY.md](SECURITY.md) rather than opening a public issue.

## License

By contributing, you agree that your contributions will be licensed under the [zlib/libpng license](LICENSE) that covers this project.
