# Conformance 验收用例

本目录定义「一份 Lemma 实现是否合格」的机械判据。共两类：

- 通过用例：[../examples/](../examples/) 下全部程序必须被接受（解析、类型检查、spec 自检全部通过）。
- 拒绝用例：[reject/](reject/) 下全部程序必须被拒绝，且拒绝原因与文件内注释一致。

## 拒绝用例清单

| 文件 | 必须违反的规则 | 预期拒绝原因 |
|---|---|---|
| reject/r1-vacuous-post.lemma | §4.3-1 非空洞 | `post true` 恒真 |
| reject/r2-unreachable-branch.lemma | §4.3-2 可达性 | 分支与 `pre` 明显矛盾 |
| reject/r3a-example-violates-pre.lemma | §4.3-3 样例一致性 | 样例输入不满足 `pre` |
| reject/r3b-example-contradicts-post.lemma | §4.3-3 样例一致性 | 样例输出与 `post` 矛盾 |
| reject/r4-undefined-reference.lemma | §4.3-4 引用合法性 | spec 引用未定义标识符 |
| reject/e1-pure-calls-io.lemma | §5 效果系统 | 纯函数调用 `io` 函数 |
| reject/n1-null-deref.lemma | §3.3 空安全 | 对 `T?` 直接字段访问 |
| reject/t1-public-no-spec.lemma | §4.4 可见性 | 公开函数缺 spec |
| reject/t2-trusted-public.lemma | §4.4 / §6 | `trusted` 用在公开函数上 |

## 通过用例清单

见 [../examples/](../examples/)：01 纯函数、02 记录与枚举、03 带 IO、04 并发、05 私有 `trusted` 辅助函数。
