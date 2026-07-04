# Security Policy

## Supported Versions

Cobogo is in early development (0.1.x). Security fixes are applied to the **latest published release** of each crate on [crates.io](https://crates.io/crates/cobogo). Older `0.1.x` versions do not receive backported fixes — please upgrade to the newest release.

| Crate | Supported |
|-------|-----------|
| `cobogo` (latest 0.1.x) | ✅ |
| `cobogo-renderer-ratatui` (latest 0.1.x) | ✅ |
| Older releases | ❌ |

## Reporting a Vulnerability

Please report security issues **privately** — do not open a public GitHub issue.

Email **joaohenriquebarbosa21@gmail.com** with the details. You can expect an initial acknowledgment within **72 hours**.

If possible, include:

- A description of the issue and its impact.
- A minimal reproduction (ideally a small `ElementDeclaration` tree or input that triggers it).
- The crate and version affected.

## Process

1. **Report received** — you email the maintainer with the details.
2. **Acknowledgment** — the maintainer confirms receipt within 72 hours.
3. **Assessment** — the issue is reproduced and its severity and scope are evaluated.
4. **Fix** — a patch is prepared and released to crates.io, with a new version published.
5. **Disclosure** — once a fix is available, the issue is disclosed and credit is given to the reporter (unless anonymity is requested).

## Scope

Cobogo is a pure-Rust layout library with no network, filesystem, or process access. Relevant security concerns are therefore about memory safety and robustness against malformed input:

- **Memory-safety or `unsafe`-block soundness issues** — any way to trigger undefined behavior, particularly through the internal structure-of-arrays storage or index bookkeeping.
- **Panics reachable from untrusted input** — a layout description, text, or pointer/scroll input that causes a panic (out-of-bounds, overflow, unwrap) rather than being handled gracefully.
- **Dependency vulnerabilities** — issues in the crates Cobogo depends on (note that the core crate has *no* runtime dependencies; the renderer depends on ratatui).

Out of scope: rendering fidelity bugs, incorrect-but-safe layout geometry, and issues in the example application (`tui-app`), which is not published. Those are welcome as regular [bug reports](.github/ISSUE_TEMPLATE/bug_report.yml).
