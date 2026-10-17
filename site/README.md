# Lemma site maintenance (gh-pages branch)

Site content is synced **only from `origin/main`**; this branch never modifies core
code (`src/`, `spec.md`, `docs/white-paper.md`, `examples/`, `conformance/`,
`Cargo.toml`, `Cargo.lock`).

## Build

    site/build.sh

The script verifies HEAD is not behind `origin/main`, extracts docs and examples
with `git show`, renders pages with pandoc, and compiles the WASM module with
`cargo rustc --crate-type cdylib`.

Dependencies: pandoc ≥ 3, rustup stable (with the `wasm32-unknown-unknown` target),
Node ≥ 20 (tests only).

## Test

    node --test site/tests/abi.test.mjs

## Updating after main advances

    git fetch origin
    git rebase origin/main
    site/build.sh
    node --test site/tests/abi.test.mjs
    git add -A && git commit -m "chore: sync origin/main and rebuild"
    git push --force-with-lease origin gh-pages

## Consistency red line

    git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock

Must be empty. Design docs live under `docs/superpowers/specs/`.
