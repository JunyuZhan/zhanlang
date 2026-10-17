# Lemma

一门为「AI 生成代码的可验证性」而设计、后被复盘转向的语言实验项目。

## 状态

早期草案。当前唯一的产物是一份复盘白皮书，而不是语言实现。项目尚未形成代码。

## 这是什么

Lemma 最初的设计目标：在语法层面强制每个函数携带可执行的意图块（`pre` / `post` / `examples`），让同一份意图同时充当规格、测试、文档与运行时断言，从而消灭 vibe coding 时代「代码、文档、测试、断言四份漂移」的成本源。

经过一次纸面原型实验后，项目结论收敛为：仅约束输入输出的点状契约，在副作用、时序与并发场景下表达成本急剧上升，且 spec 本身会空洞、自相矛盾。因此项目从「验证意图」转向两个更窄的方向：

1. 为 AI 生成的契约做静态体检（linter）：查空洞、矛盾、不可达、examples 与 post 不一致。
2. 在窄领域内用受约束构造替代事后验证。

## 目录

- [docs/white-paper.md](docs/white-paper.md) — 主白皮书（复盘报告）
- [docs/review-notes.md](docs/review-notes.md) — 对源对话与初稿的评审记录（决策依据）
- docs/experiments/ — 实验记录（待补）

## 如何参与

仓库目前处于设计阶段。先读 [docs/white-paper.md](docs/white-paper.md)，再读 [docs/review-notes.md](docs/review-notes.md) 了解哪些结论已被质疑。方向未锁死。
