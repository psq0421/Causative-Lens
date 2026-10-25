# Contributing

Keep the evidence chain short enough to audit and the prose warm enough to remember why it matters.

1. Read the relevant contract in `docs/spec/` and the task card in `docs/plan/task-cards/`.
2. Keep v0.1.0 observe-only; intervention and decision paths belong to v0.2.0.
3. Add tests before changing a contract or parser.
4. Run `cargo fmt --all -- --check`, workspace tests, and clippy with `-D warnings`.
5. Add or update all three UI translations for user-visible copy.
6. Record compatibility and measurement evidence in the gate report.

Use `[T0xxx]` commit prefixes for task-card work. Prefer small changes that can be reverted without losing the plot.

Language: [English](contributing.md) · [简体中文](contributing_CN.md) · [繁體中文](contributing_TW.md).
