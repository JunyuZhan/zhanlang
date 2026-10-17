# Changelog

本项目版本记录遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与语义化版本。

## [0.1.0] - 2026-10-17

### Added

- 语言定义：每个可见函数必须携带 `spec`，编译器同时验证「实现满足 spec」与「spec 本身非空洞、不矛盾、可达」
- 参考实现 M1–M5：词法/语法/类型检查、spec 自检、解释器与运行时断言、最小静态验证、状态词汇
- 中英双语官网（gh-pages）与在线 Playground（WASM）

### Changed

- 语言更名：Lemma → **zhanlang**（詹语言），命令 `zl`，扩展名 `.zl`

### Docs

- `spec.md`（英文规范）、`docs/white-paper.md`（英文白皮书）；中文译文 `spec.zh-CN.md`、`docs/white-paper.zh-CN.md`
