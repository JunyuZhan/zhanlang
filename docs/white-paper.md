# Lemma 语言定义

> 版本：v0.1（草案）
>
> 日期：2026-10-17
>
> 状态：语言定义，不是实现。本文是给「他人或 AI 照着实现」的规格书。

## 0. 一句话

Lemma 是一门面向 vibe coding 的编程语言：AI 写实现，人和编译器共同验证意图——而且**意图本身也是被验证的对象**。

## 1. 设计原则

本语言只服从你在最初对话里定下的四条要求，不服从任何「特性拼盘」：

1. **解决真实问题，不是拼盘。** 不为「综合各家之长」而堆特性。Lemma 只解决一个问题：代码、文档、测试、断言四份产物的漂移。
2. **AI 与 vibe coding 优先。** 把「AI 是主要代码作者、人类提供意图并审阅」当作默认前提，而不是事后补丁。
3. **第一性原理。** 先定义不可违反的不变量，再推导语法与工具链；语法和类型都为不变量服务。
4. **可被他人与 AI 实现。** 定义本身是结构化的、可执行验收的，不是散文。

## 2. 核心不变量

> **每个对外可见的函数必须携带 `spec`；`spec` 是唯一真相源；编译器在构造期同时做两件验证——验证实现满足 spec，验证 spec 本身不是空洞、矛盾或不可达的。**

这条不变量的两个半边同样重要：

- 左半边（实现 ⊨ spec）：让规格、测试、文档、断言共享一份表达。
- 右半边（spec 自检）：因为 vibe coding 里 AI 写的 spec 同样会空洞、自相矛盾，编译器必须像审查实现一样审查 spec。

早期纸面实验的教训已并入设计：spec 表达式必须能谈论状态与副作用（`old` / `modifies` / `events`），否则真实业务意图无法承载。

## 3. 程序结构

Lemma 是静态类型、表达式优先的语言。一个模块由类型声明与函数声明组成。

```lemma
module demo

type UserId = Int

type User = record {
  id: UserId
  email: String
  age: Int?
}

enum RegError {
  InvalidEmail(email: String)
  WeakPassword
  EmailTaken
}
```

类型系统只提供以下构造，不再扩：

- 基础类型：`Int`、`Float`、`Bool`、`String`。
- 空安全：`T?` 等价于 `Option<T>`。
- 代数数据类型：`enum`。
- 记录：`record`。
- 泛型：`List<T>`、`Map<K,V>`、`Result<T,E>` 等以库形式提供。
- 效果类型：见第 5 节。

非目标（刻意不做，避免拼盘）：类继承、隐式类型转换、默认动态类型、隐式效果。

## 4. 意图块 `spec`

每个对外函数必须带 `spec`。`spec` 由三类构件组成：

```lemma
fn max(a: Int, b: Int) -> Int
  spec {
    post result >= a
    post result >= b
    post result == a || result == b
    examples {
      max(1, 2) == 2
      max(5, 5) == 5
      max(0, -3) == 0
    }
  }
{
  if a >= b { a } else { b }
}
```

### 4.1 构件

- `pre`：进入函数前必须成立的条件。
- `post`：返回前必须成立的条件；用 `result` 指代返回值。
- `examples`：输入输出样例。每条样例必须是「`pre` 成立」且「被 `post` 蕴含」的，否则编译器报错。

### 4.2 状态与副作用词汇

spec 表达式默认纯，但提供三个词汇来承载真实业务意图：

- `old(expr)`：取 `expr` 在函数入口处的值。
- `modifies x`：声明本函数可写的状态，未被声明的写入即编译错误（框架约束）。
- `events`：编译器维护的 ghost 事件历史，`post events.contains(...)` 可约束「发生了什么」。

```lemma
fn register(email: String, password: String) -> Result<UserId, RegError>
  io
  spec {
    pre email.contains("@")
    pre password.len >= 8
    modifies db
    post result is Ok || result is Err(EmailTaken)
    post events.contains(EmailQueued(email)) || result is Err
    examples {
      register("a@b.com", "abcdefgh") is Ok
    }
  }
{
  // 实现
}
```

`is Ok` / `is Err(...)` 用于样例与 post 中的模式匹配断言。

### 4.3 spec 自检（右半边不变量）

编译器对每个 spec 强制执行以下检查，任一失败即拒绝编译：

1. **非空洞：** 每条 `post` 至少排除一种不合法输入或输出；`post true` 视为空洞，报错。
2. **可达性：** 每条分支在 `pre` 成立时可被触发；不可达分支报错。
3. **样例一致性：** 每条 `examples` 必须满足 `pre` 且被 `post` 蕴含。
4. **引用合法性：** spec 中出现的谓词要么是语言内置，要么是已声明且带 spec 的纯函数。

## 5. 效果系统

函数默认纯；需要外部能力的函数在签名中显式声明效果：

- `io`：读写外部状态。
- `async`：挂起与并发。
- `random`：非确定性。
- `time`：依赖时钟。

效果是类型系统的一部分：一个纯函数不能调用一个声明了 `io` 的函数。这消除了「函数看起来纯、实际偷偷发请求」这一类 vibe coding 最常见的隐式漂移。

## 6. 验证层级

构造期验证按强度三级降级，且降级必须显式、可审计：

| 层级 | 含义 | 何时使用 |
|---|---|---|
| `verified` | 编译器静态证明了 spec | 默认目标 |
| `checked` | 编译器无法静态证明，插入运行时断言 | spec 复杂或依赖外部状态时 |
| `trusted` | 显式豁免，跳过验证 | 必须有 `trusted(reason)` 标注并禁止从外部模块调用 |

降级永远只发生在「能证明的地方证明、不能证明的地方运行期检查」，不允许无声降级。

## 7. 语法摘要

核心产生式（非完整文法，v0.1 聚焦）：

```text
module    := "module" ident
decl      := type_decl | enum_decl | fn_decl
fn_decl   := "fn" ident "(" params ")" ("->" type)? effects? spec? block
spec      := "spec" "{" (pre | post | examples)* "}"
pre       := "pre" expr
post      := "post" expr
examples  := "examples" "{" (expr ("==" expr | "is" pattern))+ "}"
block     := "{" stmt* "}"
```

## 8. 最小示例集合

### 纯函数

```lemma
fn discount(price: Int, rate: Int) -> Int
  spec {
    pre price > 0
    pre 0 <= rate <= 100
    post result == (price * (100 - rate)) / 100
    post result <= price
    examples {
      discount(100, 10) == 90
      discount(1, 0) == 1
    }
  }
{
  (price * (100 - rate)) / 100
}
```

### 带 IO 的函数

```lemma
fn load_user(id: UserId) -> Result<User, RegError>
  io
  spec {
    post result is Ok || result is Err
    examples {
      load_user(1) is Ok
    }
  }
{
  // 实现
}
```

## 9. 实现路线图

| 里程碑 | 内容 | 验收标准 |
|---|---|---|
| M1 解析与类型检查 | 第 3、5、7 节的文法与类型/效果检查 | 示例程序全部通过解析与类型检查 |
| M2 spec 自检器 | 第 4.3 节的四项检查 | 构造空洞/矛盾/不可达 spec 均被拒绝 |
| M3 运行时断言 | `checked` 层级插入断言，`pre`/`post` 运行时求值 | 违反 spec 的实现在运行期被捕获 |
| M4 静态验证器 | `verified` 层级的最小符号执行（算术与布尔子集） | `max`、`discount` 被静态证明 |
| M5 状态词汇 | `old` / `modifies` / `events` 的语义与检查 | `register` 示例可写且可被检查 |

每个里程碑都有对应的 conformance 测试，任何实现（不限于官方参考实现）跑通即合格。

## 10. 与既有工作的关系

Lemma 不试图发明新的证明理论，而是把成熟构件（契约式设计、效果系统、框架约束、样例蕴含检查）以「spec 自检」为轴重新组合。真正的创新点是第 2 节那条**双半边不变量**：历史上工具要么验证实现、要么假设 spec 正确，Lemma 把「spec 本身会被 AI 写错」当作一等问题来对待。
