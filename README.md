# Lemma

一门正在设计中的编程语言，面向 vibe coding：AI 写实现，人和编译器共同验证意图，且意图本身也是被验证的对象。

## 状态

语言定义阶段（v0.1 草案）。已有语言规格书，尚无编译器实现。

## 核心不变量

每个对外可见的函数必须携带 `spec`；`spec` 是唯一真相源；编译器在构造期同时验证两件事——实现满足 spec，以及 spec 本身不是空洞、矛盾或不可达的。

## 目录

- [docs/white-paper.md](docs/white-paper.md) — 白皮书（为什么做这门语言）
- [docs/spec.md](docs/spec.md) — 语言规格书（完整语法与语义定义）
- [docs/decisions.md](docs/decisions.md) — 源对话结论纪要
- [docs/review-notes.md](docs/review-notes.md) — 早期评审与决策依据
- [examples/](examples/) — 示例程序
- [conformance/](conformance/) — 实现验收用例
- docs/experiments/ — 实验记录（待补）

## 路线图

M1 解析与类型检查 → M2 spec 自检器 → M3 运行时断言 → M4 最小静态验证 → M5 状态词汇。详见规格书第 9 节。

## 如何参与

先读 [docs/white-paper.md](docs/white-paper.md) 了解为什么，再读 [docs/spec.md](docs/spec.md) 了解语言本身。实现与语言定义以规格书为准。
