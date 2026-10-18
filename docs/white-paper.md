# zhanlang Whitepaper

> Version: v0.1
>
> Date: 2026-10-17
>
> This document explains *why* the language exists. The full syntax and semantics are in [spec.md](https://github.com/JunyuZhan/zhanlang/blob/main/spec.md).

中文译文见 [white-paper.zh-CN.md](https://github.com/JunyuZhan/zhanlang/blob/main/docs/white-paper.zh-CN.md)。

## One sentence

zhanlang is a programming language for vibe coding: AI writes the implementation, humans and the compiler verify intent — and the intent itself is verified too.

## The problem we solve

Vibe coding turns the human from "writer" into "intent provider and reviewer". That shift exposes a structural gap: the typical failure modes of AI-generated code — hallucinated APIs, off-by-one boundaries, drifting from intent — cannot be caught during generation.

Existing feedback channels (type systems, tests, runtime assertions, documentation) all run after generation and are maintained independently. Express the same intent four times and you get four copies that drift. zhanlang answers one intuition: if intent is expressed exactly once, and that one expression serves as spec, test, documentation, and assertion at the same time, does the drift disappear?

## First principles and design principles

The language obeys four requirements and nothing resembling a "feature collage":

1. Solve a real problem, don't collage. No piling features on "to borrow the best of everything"; only one problem — the drift between four artifacts.
2. AI and vibe coding first. "AI is the primary author, humans provide and review intent" is the default premise.
3. First principles. Define an inviolable invariant first, then derive syntax and tooling; syntax and types serve the invariant.
4. Implementable by others and by AI. The definition is structured and executable-acceptable, not prose.

## Core invariant

> Every publicly visible function must carry a `spec`; the `spec` is the single source of truth; at construction time the compiler does two checks — that the implementation satisfies the spec, and that the spec itself is not vacuous, contradictory, or unreachable.

The two halves of the invariant matter equally:

- Left half (implementation ⊨ spec): spec, tests, docs, and assertions share one expression.
- Right half (spec self-check): because AI-written specs are themselves vacuous or contradictory, the compiler must review the spec as rigorously as the implementation.

Lessons from the early paper prototype are folded into the design: spec expressions must be able to talk about state and side effects (`old` / `modifies` / `events`), otherwise real business intent cannot be carried.

## What zhanlang is not

Deliberately absent, to avoid a collage: class inheritance, implicit conversion, dynamic-by-default typing, implicit effects, macros, multiple dispatch. Each omission corresponds to a feature other languages already have but that has nothing to do with zhanlang's core problem.

## Roadmap

| Milestone | Scope | Acceptance |
|---|---|---|
| M1 Parsing & type checking | grammar plus type/effect checking | all example programs pass |
| M2 `spec` self-checker | non-vacuity, reachability, example consistency, reference legality | constructed vacuous/contradictory/unreachable specs are rejected |
| M3 Runtime assertions | `checked` tier inserts assertions | spec violations are caught at runtime |
| M4 Static verifier | minimal symbolic execution of the `verified` tier (linear/structural subset + if-branch expansion) | `max` is statically proved; nonlinear/recursive functions fall back to `checked` |
| M5 State vocabulary | placement rules and runtime semantics for `old` / `modifies` / `events` | examples 03 and 06 can be written and checked |

Every milestone has matching conformance tests; any implementation (not just the official reference) is conformant once those pass.

## Relation to prior work

zhanlang does not invent a new proof theory. It recombines mature pieces — design by contract, effect systems, frame constraints, example-implication checking — around the axis of "spec self-check". The real novelty is the two-sided invariant: historically, tools either verify the implementation or assume the spec is correct; zhanlang treats "the spec itself may be written wrong by an AI" as a first-class problem.
