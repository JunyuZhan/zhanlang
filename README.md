# Lemma

一门面向 vibe coding 的编程语言：AI 写实现，人和编译器共同验证意图，且意图本身也是被验证的对象。

## 状态

语言规格书 v0.1 已完成；参考实现（编译器）已起步，M1 解析与类型检查可用。

## 核心不变量

每个对外可见的函数必须携带 `spec`；`spec` 是唯一真相源；编译器在构造期同时验证两件事——实现满足 spec，以及 spec 本身不是空洞、矛盾或不可达的。

## 目录

- [spec.md](spec.md) — 语言规格书（语法与语义的规范定义）
- [docs/white-paper.md](docs/white-paper.md) — 白皮书（设计动机与第一性原理）
- [examples/](examples/) — 示例程序
- [conformance/](conformance/) — 实现验收用例

## 路线图

M1 解析与类型检查 → M2 spec 自检器 → M3 运行时断言 → M4 最小静态验证 → M5 状态词汇。详见 [docs/white-paper.md](docs/white-paper.md) 的路线图一节。

## 参与

实现以 [spec.md](spec.md) 为准；提交前先跑通 [examples/](examples/) 与 [conformance/](conformance/)。

## 构建与运行

需要 Rust 工具链。

```
cargo build
./target/debug/lemma examples/01-basics.lemma
```

已实现：词法、语法、类型检查、效果检查、空安全、spec 自检（非空洞、引用合法性、可见性与 `trusted` 规则）。

待实现：spec 样例一致性求值（M2）、分支可达性（M2）、运行时断言（M3）、最小静态验证（M4）。
