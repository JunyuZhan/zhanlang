# Lemma

A programming language for vibe coding: AI writes the implementation, humans and the compiler verify intent — and the intent itself is verified too.

中文说明见 [README.zh-CN.md](README.zh-CN.md)。

## Status

Language specification v0.1 is complete; the reference implementation (compiler) has shipped milestones M1–M5.

## Core invariant

Every publicly visible function must carry a `spec`. The `spec` is the single source of truth; at construction time the compiler verifies two things — that the implementation satisfies the spec, and that the spec itself is not vacuous, contradictory, or unreachable.

## Layout

- [spec.md](spec.md) — the language specification (normative syntax and semantics)
- [docs/white-paper.md](docs/white-paper.md) — the whitepaper (motivation and first principles)
- [examples/](examples/) — example programs
- [conformance/](conformance/) — implementation conformance cases

## Roadmap

M1 parsing & type checking → M2 spec self-checker → M3 runtime assertions → M4 minimal static verification → M5 state vocabulary. See the roadmap section in [docs/white-paper.md](docs/white-paper.md).

## Contributing

The [spec.md](spec.md) is normative for any implementation. Run [examples/](examples/) and [conformance/](conformance/) before submitting changes.

## Build & run

Requires the Rust toolchain.

```
cargo build
./target/debug/lemma examples/01-basics.lemma
./target/debug/lemma run examples/01-basics.lemma
./target/debug/lemma verify examples/01-basics.lemma
```

Implemented: lexing, parsing, type checking, effect checking, null safety, spec self-checking (non-vacuity, reference legality, visibility & `trusted` rules), M2 example consistency & reachability, M3 interpreter with runtime `pre`/`post` assertions, M4 minimal static verification (`max` is statically proved; nonlinear/recursive functions fall back to `checked`), M5 state vocabulary (`old` / `events` / `modifies` placement rules and runtime semantics).

All five milestones (M1–M5) are complete. Next: model external state and effects (databases, IO) so `modifies` / `events` become executable in real side effects.
