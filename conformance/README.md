# Conformance 验收用例

本目录定义「一份 zhanlang 实现是否合格」的机械判据。共两类：

- 通过用例：[../examples/](../examples/) 下全部程序必须被接受（解析、类型检查、spec 自检全部通过）。
- 拒绝用例：[reject/](reject/) 下全部程序必须被拒绝，且拒绝原因与文件内注释一致。

## 拒绝用例清单

| 文件 | 必须违反的规则 | 预期拒绝原因 |
|---|---|---|
| reject/r1-vacuous-post.zl | §4.3-1 非空洞 | `post true` 恒真 |
| reject/r2-unreachable-branch.zl | §4.3-2 可达性 | 分支与 `pre` 明显矛盾 |
| reject/r3a-example-violates-pre.zl | §4.3-3 样例一致性 | 样例输入不满足 `pre` |
| reject/r3b-example-contradicts-post.zl | §4.3-3 样例一致性 | 样例输出与 `post` 矛盾 |
| reject/r4-undefined-reference.zl | §4.3-4 引用合法性 | spec 引用未定义标识符 |
| reject/e1-pure-calls-io.zl | §5 效果系统 | 纯函数调用 `io` 函数 |
| reject/n1-null-deref.zl | §3.3 空安全 | 对 `T?` 直接字段访问 |
| reject/t1-public-no-spec.zl | §4.4 可见性 | 公开函数缺 spec |
| reject/t2-trusted-public.zl | §4.4 / §6 | `trusted` 用在公开函数上 |
| reject/r5-old-in-pre.zl | §4.2 状态词汇 | `old` 用在 pre 中 |
| reject/r6-events-in-body.zl | §4.2 状态词汇 | `events` 用在函数体中 |

## 通过用例清单

见 [../examples/](../examples/)：01 纯函数、02 记录与枚举、03 带 IO、04 并发、05 私有 `trusted` 辅助函数、06 `old` 语义。
