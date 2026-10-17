# Lemma

一门面向 vibe coding 的编程语言：AI 写实现，人和编译器共同验证意图，且意图本身也是被验证的对象。

## 状态

语言规格书 v0.1 已完成，编译器尚未开始。当前仓库处于「先文档、后实现」的第一阶段。

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
