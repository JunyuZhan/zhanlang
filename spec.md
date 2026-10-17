# Lemma 语言规格书

> 版本：v0.1
>
> 日期：2026-10-17
>
> 本文是 Lemma 的规范性定义。实现与本文冲突时以本文为准；本文未定义之处即「未指定」，实现可自行决定但必须在文档中记录。

## 目录

1. 词法
2. 语法
3. 类型系统
4. 意图块 spec
5. 效果系统
6. 验证层级
7. 标准库（最小）
8. 示例
9. 验收（conformance）

## 1. 词法

### 1.1 注释

- `//` 到行尾为行注释。
- `/*` 到最近的 `*/` 为块注释，不嵌套。

### 1.2 标识符与关键字

- 标识符：`[A-Za-z_][A-Za-z0-9_]*`。
- 关键字（保留，不能作标识符）：

  `module type record enum fn spec pre post examples if else match is old modifies events let return todo true false private Int Float Bool String`

- 大小写敏感。按惯例，类型与构造子以大写开头，变量与函数以小写开头，但语法不强制。

### 1.3 字面量

- 整数：`[0-9]+`，类型 `Int`（v0.1 实现用 64 位有符号整数，溢出行为由实现记录）。
- 浮点：`[0-9]+ "." [0-9]+`，类型 `Float`。
- 字符串：`"..."`，转义序列仅 `\"`、`\\`、`\n`、`\t`，类型 `String`。
- 布尔：`true`、`false`。

### 1.4 运算符与分隔符

- 运算符：`+ - * / % == != < <= > >= && || !`
- 分隔符：`( ) { } [ ] < > , . : -> => ? _`

## 2. 语法

### 2.1 程序结构

```text
program     := module_decl? decl*
module_decl := "module" ident
decl        := type_decl | enum_decl | fn_decl
```

一个文件即一个模块；`module` 可省略，省略时模块名为文件名。

### 2.2 类型声明

```text
type_decl := "type" ident "=" type
enum_decl := "enum" ident "{" variant ("," variant)* "}"
variant   := ident ("(" ident ":" type ("," ident ":" type)* ")")?
```

### 2.3 函数声明

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

可见性与 spec 的关系见 §4.4。

### 2.4 spec 块

```text
spec_block := "spec" "{" spec_item* "}"
spec_item  := "pre" expr | "post" expr
            | "modifies" ident ("," ident)*
            | "examples" "{" example* "}"
example    := expr ("==" expr | "is" pattern)
```

### 2.5 语句与块

```text
block := "{" stmt* expr? "}"
stmt  := "let" ident (":" type)? "=" expr ";"
       | "return" expr? ";"
       | expr ";"
```

块的值为其末尾表达式；无末尾表达式时块值为 `()`（单元，v0.1 不把单元当普通类型使用，只允许出现在 `return;` 语义中）。

### 2.6 类型

```text
type     := "Int" | "Float" | "Bool" | "String"
          | ident type_args?
          | type "?"
type_args:= "<" type ("," type)* ">"
```

`T?` 是 `Option<T>` 的语法糖。

### 2.7 表达式

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

优先级从低到高：`||` → `&&` → 比较 → 加减 → 乘除余 → 一元 `! -` → 后缀（调用/字段/索引）→ 主表达式。

`ident "(" ... ")"` 同时用于普通函数调用与构造子（`Ok(x)`、`Some(x)`、枚举变体）。

### 2.8 模式

```text
pattern := "_" | literal | ident ("(" pattern ("," pattern)* ")")?
```

模式出现在 `match` 分支与 `is` 断言中；`ident(p1, ...)` 是构造子模式，`ident` 单独出现是通配绑定。

## 3. 类型系统

### 3.1 类型分类

- 基础类型：`Int`、`Float`、`Bool`、`String`。
- 可选：`T?`。
- 参数化类型：`List<T>`、`Map<K, V>`、`Result<T, E>`。
- 记录：`record`。
- 枚举：`enum`。
- 类型别名：`type X = T`。

### 3.2 规则

- 静态、强类型，无隐式转换。`Int` 与 `Float` 之间必须显式转换：`Int` 提供 `to_float() -> Float`，`Float` 提供 `to_int() -> Int`（向零截断）。浮点表示带来的往返精度损失由实现记录，不保证精确。
- 泛型不变（invariant），无上界。
- `T?` 必须先解包再当作 `T` 使用；解包方式是 `match` 或 `is` 断言绑定。
- `Result<T, E>` 的合法构造子是 `Ok(T)` 与 `Err(E)`。

### 3.3 空安全

任何对 `T?` 的直接字段访问或运算都是类型错误。唯一解包途径是 `match`：

```lemma
match u.age {
  Some(a) => a >= 18
  None => false
}
```

## 4. 意图块 spec

### 4.1 构件

- `pre expr`：进入函数前必须成立的条件。
- `post expr`：返回前必须成立的条件，用 `result` 指代返回值。
- `examples`：输入输出样例，每条形如 `expr == expr` 或 `expr is pattern`。
- `modifies x, ...`：声明本函数可写的外部状态（框架约束）。

### 4.2 表达式域

spec 表达式默认纯。允许以下特殊形式：

- `old(expr)`：取 `expr` 在函数入口处的值。
- `events`：编译器维护的 ghost 事件历史，只出现在 `post` 中。
- `is`：模式匹配断言，如 `result is Ok`、`result is Err(e)`、`u.age is Some(a)`。

`body` 中不允许出现 `old`、`events`、`is`；`spec` 中不允许出现 `io` 等有副作用调用。

### 4.3 spec 自检（构造期，强制）

编译器对每个 spec 执行以下检查，任一失败即拒绝编译：

1. **非空洞。** `post` 不得是恒真式。至少拒绝 `post true` 与 `post x == x`；更一般的恒真检测为「尽力而为」，未检出不作为错误。
2. **可达性。** 对 `match result { ... }` 形式的 `post`，每个变体在 `pre` 成立时须可达；与 `pre` 明显矛盾的分支（如 `pre price > 0` 时出现 `Err(InvalidPrice) => price <= 0`）报错。完全可达性不可判定，v0.1 只做明显矛盾检测。
3. **样例一致性。** 每条 example 的输入须满足 `pre`；期望输出不得与 `post` 矛盾。纯子集内可静态判定；其余在 `checked` 层运行时判定。
4. **引用合法性。** spec 中出现的标识符必须是：参数、`result`、内置纯函数，或已声明且带 spec 的纯函数；未定义即报错。

### 4.4 可见性与省略 spec

- 顶层函数默认公开（对外可见）。
- 公开函数必须带 `spec`。
- `private` 函数可带 `spec`，也可以省略；省略时必须标注 `trusted("原因")`。
- `trusted` 只允许出现在 `private` 函数上。

## 5. 效果系统

函数默认纯。需要外部能力的函数声明效果：

- `io`：读写外部状态。
- `async`：挂起与并发。
- `random`：非确定性。
- `time`：依赖时钟。

效果沿调用图向上传播：纯函数调用声明了 `io` 的函数是类型错误。效果是函数类型的一部分。

## 6. 验证层级

构造期验证按强度三级，降级必须显式：

| 层级 | 含义 | 规则 |
|---|---|---|
| `verified` | 编译器静态证明 spec | 默认目标 |
| `checked` | 静态证不了，插入运行时断言 | 默认兜底 |
| `trusted` | 显式豁免 | 写作 `trusted("原因")`；只允许出现在 `private` 且无 spec 的函数上，外部模块调用即错误 |

降级顺序固定为「先试 verified，失败落 checked」，不允许无声降级。任何一层都不会改变 spec 的语义，只改变验证发生的时间。

## 7. 标准库（最小）

v0.1 只保证以下内置，其余按未指定。一阶函数与 `map` / `filter` 等高阶操作 v0.1 不保证。

- `Int`：`to_float() -> Float`。
- `Float`：`to_int() -> Int`（向零截断）。
- `String`：`len -> Int`、`contains(String) -> Bool`、`starts_with(String) -> Bool`、`ends_with(String) -> Bool`。
- `List<T>`：`len -> Int`、`is_empty() -> Bool`、`get(Int) -> T?`。
- `Map<K,V>`：`len -> Int`、`has(K) -> Bool`、`get(K) -> V?`。
- `Option<T>`：构造子 `Some` / `None`。
- `Result<T,E>`：构造子 `Ok` / `Err`。
- `events`：ghost 序列，`len -> Int`、`contains(Event) -> Bool`。

## 8. 示例

可运行的示例见仓库 [examples/](examples/)。核心两个：

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

## 9. 验收（conformance）

任何实现视为合格，当且仅当：

1. [examples/](examples/) 下全部程序通过解析、类型检查与 spec 自检；
2. [conformance/reject/](conformance/reject/) 下全部程序被拒绝，且拒绝原因与标注一致；
3. `checked` 层在运行期捕获违反 `pre` / `post` 的调用；
4. `trusted` 对外调用被拒绝。

conformance 用例随仓库维护，是「实现是否合格」的唯一判据。详细清单见 [conformance/README.md](conformance/README.md)。
