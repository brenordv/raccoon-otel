# Contributing
## Philosophy
Each tool follows a few principles I try to stick to:
- **Do one thing well**: Each tool has a focused purpose
- **Reasonable defaults**: Should work out of the box for common cases
- **Graceful handling**: Proper error messages and cleanup
- **Performance matters**: Use async I/O and parallel processing where it makes sense

## Coding principles
- Favor small, well-factored modules and explicit types over cleverness.
- Respect existing patterns; follow the repo’s conventions over personal preference.
- Functions ≲ ~50 LOC when feasible; extract pure helpers for parsing, graph building, and process execution.
- Never use nested ternaries (Rust’s `if/else if/else` or match statements keep control flow clear).
- Avoid `unwrap`/`expect` in library code; return typed errors. Use `?` (from `anyhow`) for propagation and convert to a single error type at the boundary.
- Use the repo’s `edition` from `Cargo.toml`; don’t change it without approval.
- When defining the CLI options, follow the examples of the other tools (tool-*).
- Use the repo’s `rustfmt` and `clippy` settings; don’t change them.
- Fix all warnings;
- Forbid `unsafe_code` unless an explicit, justified exception is approved.
- Before creating a new tool/utility, check if it is already covered by the `shared` crate.
- When planning tests, and usages, consider edge cases, but within reason.  

## Other
1. YAGNI: Let's try to keep the code simple, adding parallelism and more complex features as the need arises.
2. Be nice.