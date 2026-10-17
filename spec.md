# Lemma Language Specification

> Version: v0.1
>
> Date: 2026-10-17
>
> This document is the normative definition of Lemma. Where an implementation conflicts with this document, this document wins; anything left undefined here is "unspecified" and an implementation may choose freely, but must document its choice.

中文译文见 [spec.zh-CN.md](spec.zh-CN.md)（非规范，以本英文版为准）。

## Table of contents

1. Lexical structure
2. Syntax
3. Type system
4. Intent blocks (`spec`)
5. Effect system
6. Verification tiers
7. Standard library (minimal)
8. Examples
9. Conformance

## 1. Lexical structure

### 1.1 Comments

- `//` to end of line is a line comment.
- `/*` to the nearest `*/` is a block comment; it does not nest.

### 1.2 Identifiers and keywords

- Identifier: `[A-Za-z_][A-Za-z0-9_]*`.
- Keywords (reserved; cannot be used as identifiers):

  `module type record enum fn spec pre post examples if else match is old modifies events let return todo true false private Int Float Bool String`

- Case-sensitive. By convention types and constructors start with an uppercase letter and variables and functions with a lowercase letter, but the grammar does not enforce this.

### 1.3 Literals

- Integer: `[0-9]+`, type `Int` (v0.1 uses 64-bit signed integers; overflow behavior is implementation-documented).
- Float: `[0-9]+ "." [0-9]+`, type `Float`.
- String: `"..."`; the only escapes are `\"`, `\\`, `\n`, `\t`; type `String`.
- Boolean: `true`, `false`.

### 1.4 Operators and delimiters

- Operators: `+ - * / % == != < <= > >= && || !`
- Delimiters: `( ) { } [ ] < > , . : -> => ? _`

## 2. Syntax

### 2.1 Program structure

```text
program     := module_decl? decl*
module_decl := "module" ident
decl        := type_decl | enum_decl | fn_decl
```

A file is a module; `module` may be omitted, in which case the module name is the file name.

### 2.2 Type declarations

```text
type_decl := "type" ident "=" type
enum_decl := "enum" ident "{" variant ("," variant)* "}"
variant   := ident ("(" ident ":" type ("," ident ":" type)* ")")?
```

### 2.3 Function declarations

```text
fn_decl    := "private"? "fn" ident type_params? "(" params? ")" ("->" type)? effects? body_part? block
type_params:= "<" ident ("," ident)* ">"
params     := param ("," param)*
param      := ident ":" type
effects    := effect+
effect     := "io" | "async" | "random" | "time"
body_part  := spec_block | trusted_note
trusted_note := "trusted" "(" string ")"
```

Visibility and its relation to `spec` are described in §4.4.

### 2.4 `spec` blocks

```text
spec_block := "spec" "{" spec_item* "}"
spec_item  := "pre" expr | "post" expr
            | "modifies" ident ("," ident)*
            | "examples" "{" example* "}"
example    := expr ("==" expr | "is" pattern)
```

### 2.5 Statements and blocks

```text
block := "{" stmt* expr? "}"
stmt  := "let" ident (":" type)? "=" expr ";"
       | "return" expr? ";"
       | expr ";"
```

The value of a block is its trailing expression; without one, the block yields `()` (unit). v0.1 does not treat unit as an ordinary type; it exists only for `return;`.

### 2.6 Types

```text
type     := "Int" | "Float" | "Bool" | "String"
          | ident type_args?
          | type "?"
type_args:= "<" type ("," type)* ">"
```

`T?` is sugar for `Option<T>`.

### 2.7 Expressions

```text
expr       := if_expr | match_expr | or
if_expr    := "if" expr block "else" block
match_expr := "match" expr "{" match_arm ("," match_arm)* "}"
match_arm  := pattern "=>" expr
or         := and ("||" and)*
and        := cmp ("&&" cmp)*
cmp        := add (("=="|"!="|"<"|"<="|">"|">=") add)*
add        := mul (("+"|"-") mul)*
mul        := unary (("*"|"/"|"%") unary)*
unary      := "-" unary | "!" unary | postfix
postfix    := primary ("(" expr ("," expr)* ")" | "." ident | "[" expr "]")*
primary    := literal | ident | "(" expr ")" | "todo"
            | ident "{" field ("," field)* "}"      // 记录字面量
field      := ident ":" expr
```

Precedence, low to high: `||` → `&&` → comparison → addition/subtraction → multiplication/division/remainder → unary `! -` → postfix (call/field/index) → primary.

`ident "(" ... ")"` covers ordinary function calls as well as constructors (`Ok(x)`, `Some(x)`, enum variants).

### 2.8 Patterns

```text
pattern := "_" | literal | ident ("(" pattern ("," pattern)* ")")?
```

Patterns appear in `match` arms and in `is` assertions. `ident(p1, ...)` is a constructor pattern; a bare `ident` is a wildcard binding.

## 3. Type system

### 3.1 Classification

- Primitive types: `Int`, `Float`, `Bool`, `String`.
- Optional: `T?`.
- Parameterized types: `List<T>`, `Map<K, V>`, `Result<T, E>`.
- Records: `record`.
- Enums: `enum`.
- Type aliases: `type X = T`.

### 3.2 Rules

- Static and strongly typed, no implicit conversion. `Int` and `Float` must be converted explicitly: `Int` has `to_float() -> Float` and `Float` has `to_int() -> Int` (truncating toward zero). Round-trip precision loss from floating-point representation is implementation-documented and not guaranteed.
- Generics are invariant, with no bounds.
- A `T?` must be unwrapped before it is used as `T`; the only ways to unwrap are `match` and `is`-assertion binding.
- The legal constructors of `Result<T, E>` are `Ok(T)` and `Err(E)`.

### 3.3 Null safety

Any direct field access or operation on `T?` is a type error. The only way to unwrap is `match`:

```lemma
match u.age {
  Some(a) => a >= 18
  None => false
}
```

## 4. Intent blocks (`spec`)

### 4.1 Components

- `pre expr`: a condition that must hold on entry.
- `post expr`: a condition that must hold on return; `result` names the return value.
- `examples`: input/output examples, each of the form `expr == expr` or `expr is pattern`.
- `modifies x, ...`: declares the external state this function may write (frame constraint).

### 4.2 Expression domain

`spec` expressions are pure by default. The following special forms are allowed:

- `old(expr)`: the value of `expr` at function entry; allowed only in `post`.
- `events`: a compiler-maintained ghost event history; allowed only in `post`.
- `is`: a pattern-match assertion, e.g. `result is Ok`, `result is Err(e)`, `u.age is Some(a)`.

`old`, `events`, and `is` are not allowed in a function body; side-effecting calls such as `io` are not allowed in `spec`.

### 4.3 `spec` self-check (at construction time, mandatory)

The compiler performs the following checks on every `spec`; any failure rejects compilation:

1. **Non-vacuity.** A `post` must not be a tautology. At minimum `post true` and `post x == x` are rejected; more general tautology detection is best-effort and a miss is not an error.
2. **Reachability.** For a `post` of the form `match result { ... }`, every variant must be reachable when `pre` holds; a branch that obviously contradicts `pre` (e.g. `Err(InvalidPrice) => price <= 0` while `pre price > 0`) is rejected. Full reachability is undecidable; v0.1 detects only obvious contradictions.
3. **Example consistency.** Each example's inputs must satisfy `pre`, and its expected output must not contradict `post`. This is decided statically within the pure subset and at runtime in the `checked` tier otherwise.
4. **Reference legality.** Every identifier in `spec` must be a parameter, `result`, a built-in pure function, or a declared pure function that carries its own `spec`; undefined identifiers are rejected.

### 4.4 Visibility and omitting `spec`

- Top-level functions are public (externally visible) by default.
- Public functions must carry a `spec`.
- `private` functions may carry a `spec` or omit it; omission requires a `trusted("reason")` annotation.
- `trusted` is allowed only on `private` functions.

## 5. Effect system

Functions are pure by default. Functions that need external capabilities declare effects:

- `io`: reads or writes external state.
- `async`: suspension and concurrency.
- `random`: non-determinism.
- `time`: dependence on the clock.

Effects propagate up the call graph: a pure function calling a function that declares `io` is a type error. Effects are part of a function's type.

## 6. Verification tiers

Construction-time verification has three tiers; downgrading must be explicit:

| Tier | Meaning | Rule |
|---|---|---|
| `verified` | the compiler statically proves the `spec` | default target |
| `checked` | not statically provable; runtime assertions are inserted | default fallback |
| `trusted` | explicit exemption | written `trusted("reason")`; allowed only on `private` functions without a `spec`; calling it from another module is an error |

The downgrade order is fixed: try `verified`, fall back to `checked`. Silent downgrade is not allowed. No tier changes the meaning of the `spec`, only when verification happens.

## 7. Standard library (minimal)

v0.1 guarantees only the following built-ins; everything else is unspecified. First-class functions and higher-order operations such as `map` / `filter` are not guaranteed in v0.1.

- `Int`: `to_float() -> Float`.
- `Float`: `to_int() -> Int` (truncating toward zero).
- `String`: `len -> Int`, `contains(String) -> Bool`, `starts_with(String) -> Bool`, `ends_with(String) -> Bool`.
- `List<T>`: `len -> Int`, `is_empty() -> Bool`, `get(Int) -> T?`.
- `Map<K,V>`: `len -> Int`, `has(K) -> Bool`, `get(K) -> V?`.
- `Option<T>`: constructors `Some` / `None`.
- `Result<T,E>`: constructors `Ok` / `Err`.
- `events`: a ghost sequence, `len -> Int`, `contains(Event) -> Bool`.

## 8. Examples

Runnable examples live in [examples/](examples/). Two core ones:

```lemma
fn max(a: Int, b: Int) -> Int
  spec {
    post result >= a
    post result >= b
    post result == a || result == b
    examples {
      max(1, 2) == 2
      max(5, 5) == 5
    }
  }
{
  if a >= b { a } else { b }
}
```

```lemma
fn register(email: String, password: String) -> Result<UserId, RegError>
  io
  spec {
    pre email.contains("@")
    pre password.len >= 8
    modifies db
    post result is Ok || result is Err(EmailTaken)
    examples {
      register("a@b.com", "abcdefgh") is Ok
    }
  }
{
  todo
}
```

## 9. Conformance

An implementation is conformant if and only if:

1. every program under [examples/](examples/) passes parsing, type checking, and `spec` self-check;
2. every program under [conformance/reject/](conformance/reject/) is rejected for the annotated reason;
3. the `checked` tier catches calls that violate `pre` / `post` at runtime;
4. calling a `trusted` function from outside its module is rejected.

The conformance cases are maintained alongside the repository and are the sole judge of conformance. See [conformance/README.md](conformance/README.md).
