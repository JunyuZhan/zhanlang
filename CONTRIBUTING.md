# Contributing

Thanks for your interest in zhanlang.

## Getting started

- Read [spec.md](spec.md) — it is normative for any implementation.
- Run `cargo test` before submitting changes.

## Submitting changes

1. Open an issue for bugs or feature ideas first.
2. Create a branch off `main`.
3. Keep the spec and the implementation in sync: any behavior change must update `spec.md` and the conformance cases under `conformance/`.
4. Add tests; run `cargo test` and make sure everything passes.
5. Submit a pull request against `main`.

## Language policy

The English [spec.md](spec.md) is the normative text. Chinese translations under `spec.zh-CN.md` are informative only.
