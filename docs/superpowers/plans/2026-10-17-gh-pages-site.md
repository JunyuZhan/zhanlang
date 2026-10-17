# Lemma 官网（gh-pages）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 gh-pages 分支交付 Lemma 官网：Vercel 风格首页、规格书/白皮书 HTML、WASM 在线 playground，内容只从 `origin/main` 同步。

**Architecture:** worktree `../Lemma-pages`（分支 `gh-pages`，基点 `origin/main`）保留完整 Rust 源码树，站点文件叠加在根目录。`site/build.sh` 用 `git show origin/main:<path>` 提取文档与示例，pandoc 渲染，`cargo rustc --crate-type cdylib` 编译 WASM（现有 C ABI，无 wasm-bindgen）。浏览器端 `assets/lemma-abi.mjs` 封装 WASM 调用（Node 可测），`assets/play.js` 只做 DOM。

**Tech Stack:** bash、pandoc ≥ 3、Rust（rustup stable + wasm32-unknown-unknown）、原生 ES modules、Node ≥ 20（仅测试）、hand-written CSS（Vercel tokens，fontsource CDN 字体）。

**Spec:** `docs/superpowers/specs/2026-10-17-gh-pages-site-design.md`（本计划所有要求均源自该 spec；执行者先读两者）

## Global Constraints

- 一致性红线：`git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock` 必须始终为空；**不修改**上述任何核心文件
- 所有站点内链接与资源引用一律**相对路径**（GitHub Pages 项目站根为 `/Lemma/`，相对路径天然兼容自定义域名与本地预览）；外部链接（GitHub、CDN）用绝对 URL
- 站点 UI 文案、文档内容均为中文；HTML `lang="zh-CN"`
- 视觉 tokens（Vercel 参考）：画布 `#fafafa`、卡片 `#ffffff`、发丝线 `#ebebeb`、主文本 `#171717`、正文 `#4d4d4d`、弱化 `#666666`、唯一彩色 Terminal Green `#297a3a`；卡片/按钮圆角 6px；深度只用发丝线 ring，禁用投影；display 级标题 weight 450、字距 -0.06em、line-height 1
- 字体 CDN（已验证 200）：`https://cdn.jsdelivr.net/npm/@fontsource-variable/geist/index.css` 与 `https://cdn.jsdelivr.net/npm/@fontsource-variable/geist-mono/index.css`；Geist 不含 CJK，中文走系统字体栈（fallback 必须含 `system-ui`）
- push 一律显式 `git push origin gh-pages`（分支跟踪了 origin/main，默认 push 会误推 main）
- 每个任务的验证命令都在 worktree `/Users/apple/Documents/Project/Lemma-pages` 内执行
- 已知环境事实：Homebrew cargo/rustc 在 PATH 优先且 `rustup run` 不覆盖 PATH，构建前必须前置 rustup 工具链目录（见 Task 3）

---

### Task 1: 主题样式与文档模板

**Files:**
- Create: `assets/theme.css`
- Create: `site/templates/doc.html`

**Interfaces:**
- Consumes: 无（首个任务）
- Produces: CSS class 契约供后续页面使用——`.nav`、`.btn`、`.btn-primary`、`.btn-ghost`、`.card-grid`、`.card`、`.eyebrow`、`.doc-layout`、`.doc-toc`、`.doc-content`、`.hero`、`.footer`、`.code-block`、`pre code`、`.play-layout`、`.play-editor`、`.play-output`、`.ok`、`.err`；pandoc 模板变量 `$title$`、`$body$`、`$toc$`

- [ ] **Step 1: 写 `assets/theme.css`（完整内容）**

```css
/* Lemma 官网主题 — Vercel 风格 tokens（见 spec「视觉风格」） */
:root {
  --paper: #fafafa;
  --card: #ffffff;
  --hairline: #ebebeb;
  --ink: #171717;
  --body: #4d4d4d;
  --muted: #666666;
  --green: #297a3a;
  --radius: 6px;
  --sans: "Geist Variable", "Geist", ui-sans-serif, system-ui, -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif;
  --mono: "Geist Mono Variable", "Geist Mono", ui-monospace, "SF Mono", Menlo, Consolas, monospace;
  --maxw: 1280px;
  --docw: 720px;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--sans);
  font-size: 16px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}
a { color: var(--green); text-decoration: none; }
a:hover { text-decoration: underline; }
main { max-width: var(--maxw); margin: 0 auto; padding: 0 32px; }

/* 顶栏 */
.nav {
  position: sticky; top: 0; z-index: 10;
  height: 64px; display: flex; align-items: center; gap: 24px;
  max-width: var(--maxw); margin: 0 auto; padding: 0 32px;
  background: color-mix(in srgb, var(--paper) 80%, transparent);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
}
.nav .brand { color: var(--ink); font-weight: 500; font-size: 14px; }
.nav .brand:hover { text-decoration: none; }
.nav .brand .tri { color: #000; margin-right: 6px; }
.nav .links { display: flex; gap: 20px; font-size: 14px; }
.nav .links a { color: var(--ink); }
.nav .spacer { flex: 1; }

/* 按钮 */
.btn {
  display: inline-block; padding: 10px 20px; border-radius: var(--radius);
  font-size: 14px; line-height: 1.43; cursor: pointer; border: none;
  font-family: var(--sans);
}
.btn:hover { text-decoration: none; }
.btn-primary { background: var(--ink); color: #fff; }
.btn-primary:hover { background: #000; }
.btn-ghost {
  background: transparent; color: var(--body);
  box-shadow: 0 0 0 1px var(--hairline);
}
.btn-ghost:hover { color: var(--ink); }

/* eyebrow 标签 */
.eyebrow {
  font-family: var(--mono); font-size: 11px; font-weight: 400;
  letter-spacing: 0.071em; text-transform: uppercase; color: var(--ink);
  margin: 0 0 12px;
}

/* 卡片 */
.card-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; }
.card {
  background: var(--card); border-radius: var(--radius); padding: 20px;
  box-shadow: 0 0 0 1px rgba(0,0,0,0.08), 0 0 0 2px var(--paper);
}
.card h3 { margin: 0 0 8px; font-size: 18px; font-weight: 500; }
.card p { margin: 0; color: var(--body); font-size: 14px; }

/* Hero */
.hero { padding: 96px 0 64px; }
.hero h1 {
  margin: 0 0 20px; font-size: 64px; font-weight: 450;
  letter-spacing: -0.06em; line-height: 1; max-width: 18em;
}
.hero .sub { color: var(--body); font-size: 18px; margin: 0 0 32px; max-width: 36em; }
.hero .cta { display: flex; gap: 12px; }

/* 首页分区 */
.section { padding: 64px 0; }
.section h2 {
  margin: 0 0 24px; font-size: 30px; font-weight: 400;
  letter-spacing: -0.05em; line-height: 1.1;
}
.roadmap { display: flex; gap: 8px; flex-wrap: wrap; list-style: none; padding: 0; margin: 0; }
.roadmap li {
  font-family: var(--mono); font-size: 12px; letter-spacing: 0.05em;
  padding: 6px 12px; border-radius: 999px;
  box-shadow: 0 0 0 1px var(--hairline); color: var(--muted);
}
.roadmap li.current { background: var(--ink); color: #fff; box-shadow: none; }

/* 代码 */
pre {
  background: var(--card); border-radius: var(--radius); padding: 16px;
  overflow-x: auto; font-size: 13px; line-height: 1.6;
  box-shadow: 0 0 0 1px var(--hairline);
}
code { font-family: var(--mono); font-size: 0.92em; }
:not(pre) > code {
  background: var(--card); padding: 2px 5px; border-radius: 4px;
  box-shadow: 0 0 0 1px var(--hairline);
}

/* 文档页 */
.doc-layout {
  display: grid; grid-template-columns: 220px minmax(0, var(--docw));
  gap: 48px; justify-content: center; padding: 48px 32px 96px;
}
.doc-toc { position: sticky; top: 88px; align-self: start; font-size: 13px; }
.doc-toc .toc-title {
  font-family: var(--mono); font-size: 11px; letter-spacing: 0.071em;
  text-transform: uppercase; color: var(--muted); margin-bottom: 12px;
}
.doc-toc ul { list-style: none; padding: 0; margin: 0; }
.doc-toc li { margin: 6px 0; }
.doc-toc a { color: var(--body); }
.doc-toc a:hover { color: var(--ink); }
.doc-toc ul ul { padding-left: 12px; }
.doc-content h1 { font-size: 40px; font-weight: 450; letter-spacing: -0.04em; line-height: 1.1; margin: 0 0 32px; }
.doc-content h2 {
  font-size: 24px; font-weight: 500; letter-spacing: -0.02em;
  margin: 48px 0 16px; padding-top: 24px; border-top: 1px solid var(--hairline);
}
.doc-content h3 { font-size: 18px; font-weight: 500; margin: 32px 0 12px; }
.doc-content p, .doc-content li { color: var(--body); font-size: 15px; }
.doc-content li { margin: 4px 0; }
.doc-content table {
  border-collapse: collapse; width: 100%; margin: 16px 0; font-size: 14px;
}
.doc-content th, .doc-content td {
  border-bottom: 1px solid var(--hairline); padding: 8px 12px; text-align: left;
}
.doc-content th { font-weight: 500; color: var(--ink); }
.doc-content blockquote {
  margin: 16px 0; padding: 4px 20px; border-left: 3px solid var(--hairline); color: var(--muted);
}
.doc-page-title { display: none; } /* 标题已在 markdown H1 中，模板不重复渲染 */

/* Playground */
.play-layout {
  display: grid; grid-template-columns: 1fr 1fr; gap: 16px;
  padding: 32px 32px 96px; max-width: var(--maxw); margin: 0 auto;
  min-height: 60vh;
}
.play-pane { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
.play-pane label {
  font-family: var(--mono); font-size: 11px; letter-spacing: 0.071em;
  text-transform: uppercase; color: var(--muted);
}
.play-editor {
  flex: 1; min-height: 420px; resize: vertical;
  font-family: var(--mono); font-size: 13px; line-height: 1.6;
  padding: 16px; border-radius: var(--radius); border: none;
  background: var(--card); color: var(--ink);
  box-shadow: 0 0 0 1px var(--hairline);
}
.play-editor:focus { outline: none; box-shadow: 0 0 0 1px var(--ink); }
.play-output {
  flex: 1; min-height: 420px; margin: 0; padding: 16px;
  border-radius: var(--radius); background: var(--card);
  box-shadow: 0 0 0 1px var(--hairline);
  font-family: var(--mono); font-size: 13px; line-height: 1.6;
  white-space: pre-wrap; overflow-y: auto; color: var(--body);
}
.play-output .ok { color: var(--green); }
.play-output .err { color: #b00020; }
.play-select {
  font-family: var(--mono); font-size: 12px; padding: 6px 10px;
  border-radius: var(--radius); border: none; background: var(--card);
  color: var(--ink); box-shadow: 0 0 0 1px var(--hairline);
}

/* 页脚 */
.footer {
  border-top: 1px solid var(--hairline); margin-top: 64px;
  padding: 32px; max-width: var(--maxw); margin-left: auto; margin-right: auto;
  font-size: 13px; color: var(--muted);
  display: flex; gap: 24px; flex-wrap: wrap;
}
.footer a { color: var(--body); }
.footer .commit { font-family: var(--mono); font-size: 12px; }

@media (max-width: 900px) {
  .hero h1 { font-size: 40px; }
  .card-grid { grid-template-columns: 1fr; }
  .doc-layout { grid-template-columns: 1fr; gap: 24px; }
  .doc-toc { position: static; }
  .play-layout { grid-template-columns: 1fr; }
}
```

- [ ] **Step 2: 写 `site/templates/doc.html`（完整 pandoc 模板）**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>$title$ · Lemma</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist/index.css">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist-mono/index.css">
<link rel="stylesheet" href="../assets/theme.css">
</head>
<body>
<nav class="nav">
  <a class="brand" href="../"><span class="tri">▲</span>Lemma</a>
  <div class="links">
    <a href="../spec/">规格书</a>
    <a href="../white-paper/">白皮书</a>
    <a href="../play/">Playground</a>
    <a href="https://github.com/JunyuZhan/Lemma">GitHub</a>
  </div>
</nav>
<div class="doc-layout">
$if(toc)$
<aside class="doc-toc">
  <div class="toc-title">目录</div>
  $toc$
</aside>
$endif$
<article class="doc-content">
$body$
</article>
</div>
<footer class="footer">
  <a href="https://github.com/JunyuZhan/Lemma">GitHub 仓库</a>
  <span>内容与 <a href="https://github.com/JunyuZhan/Lemma">origin/main</a> 同步</span>
</footer>
</body>
</html>
```

- [ ] **Step 3: 用最小 markdown 冒烟验证模板渲染**

Run（在 worktree 根）:
```bash
mkdir -p /tmp/lemma-smoke && printf '# 冒烟\n\n段落。\n' > /tmp/lemma-smoke/t.md
pandoc /tmp/lemma-smoke/t.md --standalone --template site/templates/doc.html --toc --metadata title=冒烟 -o /tmp/lemma-smoke/t.html
grep -c 'class="nav"' /tmp/lemma-smoke/t.html && grep -c 'doc-layout' /tmp/lemma-smoke/t.html && grep -c '<h1' /tmp/lemma-smoke/t.html
```
Expected: 三条 grep 各输出 ≥1 的计数且退出码 0（模板变量无残留 `$`：`grep -c '\$[a-z]' /tmp/lemma-smoke/t.html` 应为 0）

- [ ] **Step 4: Commit**

```bash
git add assets/theme.css site/templates/doc.html
git commit -m "feat: Vercel 风格主题与文档页 pandoc 模板"
```

---

### Task 2: 构建脚本骨架与文档渲染

**Files:**
- Create: `site/build.sh`

**Interfaces:**
- Consumes: `site/templates/doc.html`（Task 1）
- Produces: 可执行 `site/build.sh`；运行后生成 `spec/index.html`、`white-paper/index.html`、`assets/examples/{01-basics,02-records,03-io,04-batch,05-private-trusted}.lemma`。后续任务在该脚本末尾追加 WASM 与首页构建段。

- [ ] **Step 1: 写 `site/build.sh`（完整内容；Task 3/5 会在文末追加段落）**

```bash
#!/usr/bin/env bash
# Lemma 官网构建：内容一律取自 origin/main，绝不读工作区副本。
set -euo pipefail
cd "$(dirname "$0")/.."

# 1) 基线校验：origin/main 必须是 HEAD 祖先（分支落后则拒绝构建）
if ! git merge-base --is-ancestor origin/main HEAD; then
  echo "错误: 分支落后于 origin/main。先执行: git fetch origin && git rebase origin/main" >&2
  exit 1
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# 2) 从 origin/main 提取内容
git show origin/main:spec.md > "$TMP/spec.md"
git show origin/main:docs/white-paper.md > "$TMP/white-paper.md"
mkdir -p assets/examples
EXAMPLES="01-basics 02-records 03-io 04-batch 05-private-trusted"
for name in $EXAMPLES; do
  git show "origin/main:examples/$name.lemma" > "assets/examples/$name.lemma"
done

# 3) 文档渲染
mkdir -p spec white-paper
pandoc "$TMP/spec.md" --standalone --template site/templates/doc.html \
  --toc --metadata title=语言规格书 -o spec/index.html
pandoc "$TMP/white-paper.md" --standalone --template site/templates/doc.html \
  --toc --metadata title=白皮书 -o white-paper/index.html

echo "文档构建完成: spec/index.html white-paper/index.html"
```

- [ ] **Step 2: 赋予执行权限并运行**

Run:
```bash
chmod +x site/build.sh && site/build.sh
```
Expected: 输出「文档构建完成」，退出码 0

- [ ] **Step 3: 验证产物内容与幂等**

Run:
```bash
grep -q '语言规格' spec/index.html && grep -q 'doc-toc' spec/index.html
grep -q '白皮书\|设计动机' white-paper/index.html
ls assets/examples/ | wc -l   # 期望 5
shasum spec/index.html white-paper/index.html > /tmp/lemma-sha1
site/build.sh
shasum -c /tmp/lemma-sha1     # 两次构建产物一致
```
Expected: 全部通过；`shasum -c` 报 OK（幂等）

- [ ] **Step 4: Commit**

```bash
git add site/build.sh
git commit -m "feat: 构建脚本——origin/main 内容提取与文档渲染"
```

---

### Task 3: WASM 构建段与 ABI 模块（TDD）

**Files:**
- Create: `assets/lemma-abi.mjs`
- Create: `site/tests/abi.test.mjs`
- Modify: `site/build.sh`（文末追加 WASM 段）

**Interfaces:**
- Consumes: `lemma_check(ptr, len) -> *mut u8`（NUL 结尾 UTF-8）、`lemma_free(ptr, len)`（len 含尾部 NUL）、导出 `memory`（均为 main 上现有 C ABI）
- Produces: `loadLemma(source: string | ArrayBuffer | Uint8Array) -> Promise<{ check(src: string): string }>`（`check` 返回 `"OK"` 或逐行错误文本）；`assets/lemma.wasm` 产物

- [ ] **Step 1: 写失败测试 `site/tests/abi.test.mjs`**

```js
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { loadLemma } from "../../assets/lemma-abi.mjs";

const wasmBytes = await readFile(
  new URL("../../assets/lemma.wasm", import.meta.url)
);
const validSrc = await readFile(
  new URL("../../assets/examples/01-basics.lemma", import.meta.url),
  "utf8"
);

test("合法示例返回 OK", async () => {
  const mod = await loadLemma(wasmBytes);
  assert.equal(mod.check(validSrc), "OK");
});

test("语法错误返回错误文本", async () => {
  const mod = await loadLemma(wasmBytes);
  const out = mod.check("module m\nfn f( ->\n");
  assert.notEqual(out, "OK");
  assert.match(out, /错误/);
});

test("连续多次检查稳定（scratch 复用 + free 约定）", async () => {
  const mod = await loadLemma(wasmBytes);
  for (let i = 0; i < 50; i++) {
    assert.equal(mod.check(validSrc), "OK");
  }
});

test("超长输入触发 memory.grow 仍正确", async () => {
  const mod = await loadLemma(wasmBytes);
  const padded = "// " + "x".repeat(200_000) + "\n" + validSrc;
  assert.equal(mod.check(padded), "OK");
});
```

- [ ] **Step 2: 在 build.sh 追加 WASM 构建段并构建**

在 `site/build.sh` 末尾（`echo "文档构建完成..."` 之后）追加：

```bash

# 4) WASM 构建（不改 Cargo.toml：命令行注入 cdylib）
export PATH="$(dirname "$(rustup which rustc)"):$(dirname "$(rustup which cargo)"):$PATH"
cargo rustc --release --lib --target wasm32-unknown-unknown -- --crate-type cdylib
cp target/wasm32-unknown-unknown/release/deps/lemma-*.wasm assets/lemma.wasm
echo "WASM 构建完成: assets/lemma.wasm"
```

Run: `site/build.sh`
Expected: 末行输出「WASM 构建完成: assets/lemma.wasm」，`ls -la assets/lemma.wasm` 约 208KB

- [ ] **Step 3: 运行测试确认失败（模块尚不存在）**

Run: `node --test site/tests/abi.test.mjs`
Expected: FAIL，报 `Cannot find module '../../assets/lemma-abi.mjs'`

- [ ] **Step 4: 实现 `assets/lemma-abi.mjs`（完整内容）**

```js
// Lemma WASM ABI 封装：浏览器与 Node 通用。
// ABI 约定（与 src/lib.rs wasm 壳一致）：
//   lemma_check(ptr, len) -> NUL 结尾 UTF-8 指针（Rust 侧分配）
//   lemma_free(ptr, len)  -> len 为含尾部 NUL 的字节数
// 输入缓冲：JS 管理的 64KB scratch 页循环复用，零泄漏。

export async function loadLemma(source) {
  let bytes;
  if (typeof source === "string") {
    const resp = await fetch(source);
    if (!resp.ok) throw new Error(`加载 WASM 失败: HTTP ${resp.status}`);
    bytes = await resp.arrayBuffer();
  } else if (source instanceof ArrayBuffer) {
    bytes = source;
  } else {
    bytes = source.buffer.slice(source.byteOffset, source.byteOffset + source.byteLength);
  }

  const { instance } = await WebAssembly.instantiate(bytes, {});
  const { lemma_check, lemma_free, memory } = instance.exports;
  if (typeof lemma_check !== "function" || typeof lemma_free !== "function") {
    throw new Error("WASM 缺少 lemma_check/lemma_free 导出");
  }

  let scratchPtr = 0;
  let scratchCap = 0;
  function ensureScratch(len) {
    if (len <= scratchCap) return scratchPtr;
    const before = memory.buffer.byteLength;
    const need = Math.max(1, Math.ceil((len - scratchCap) / 65536));
    memory.grow(need);
    scratchPtr = before;
    scratchCap = memory.buffer.byteLength - before;
    return scratchPtr;
  }

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  return {
    check(src) {
      const input = encoder.encode(src);
      const ptr = ensureScratch(Math.max(input.length, 1));
      if (input.length > 0) {
        new Uint8Array(memory.buffer, ptr, input.length).set(input);
      }
      const outPtr = lemma_check(ptr, input.length);
      // 调用后 memory 可能已 grow，必须重新取视图
      const view = new Uint8Array(memory.buffer);
      let end = outPtr;
      while (view[end] !== 0) end++;
      const text = decoder.decode(view.subarray(outPtr, end));
      lemma_free(outPtr, end - outPtr + 1);
      return text;
    },
  };
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `node --test site/tests/abi.test.mjs`
Expected: 4 tests PASS

- [ ] **Step 6: 验证一致性红线并 Commit**

Run:
```bash
git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock
```
Expected: 无输出（空 diff）

```bash
git add assets/lemma-abi.mjs site/tests/abi.test.mjs site/build.sh
git commit -m "feat: WASM 构建与 ABI 封装模块（含 Node 测试）"
```

---

### Task 4: Playground 页面

**Files:**
- Create: `assets/play.js`
- Create: `play/index.html`

**Interfaces:**
- Consumes: `loadLemma` from `../assets/lemma-abi.mjs`（Task 3）；示例文件 `assets/examples/*.lemma`（Task 2 产物）；class 契约 `.play-layout`、`.play-editor`、`.play-output`、`.play-select`、`.ok`、`.err`（Task 1）
- Produces: `/play/` 页面。无导出接口（纯页面）。

- [ ] **Step 1: 写 `assets/play.js`（完整内容）**

```js
import { loadLemma } from "./lemma-abi.mjs";

const EXAMPLES = [
  ["01-basics", "基础：纯函数与 spec"],
  ["02-records", "记录类型"],
  ["03-io", "IO 与效果"],
  ["04-batch", "批处理"],
  ["05-private-trusted", "私有函数与 trusted"],
];

const editor = document.getElementById("editor");
const output = document.getElementById("output");
const select = document.getElementById("example-select");
const status = document.getElementById("play-status");

for (const [id, label] of EXAMPLES) {
  const opt = document.createElement("option");
  opt.value = id;
  opt.textContent = label;
  select.appendChild(opt);
}

function render(text) {
  if (text === "OK") {
    output.innerHTML = '<span class="ok">✓ 检查通过</span>';
    status.textContent = "OK";
    return;
  }
  const escaped = text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  output.innerHTML = `<span class="err">${escaped}</span>`;
  status.textContent = "有错误";
}

let mod = null;
let timer = null;

async function runCheck() {
  if (!mod) return;
  try {
    render(mod.check(editor.value));
  } catch (e) {
    output.innerHTML = `<span class="err">检查失败: ${e.message}</span>`;
    status.textContent = "失败";
  }
}

function scheduleCheck() {
  clearTimeout(timer);
  timer = setTimeout(runCheck, 300);
}

editor.addEventListener("input", scheduleCheck);

select.addEventListener("change", async () => {
  const resp = await fetch(`../assets/examples/${select.value}.lemma`);
  editor.value = await resp.text();
  runCheck();
});

(async () => {
  status.textContent = "加载 WASM…";
  try {
    mod = await loadLemma("../assets/lemma.wasm");
    status.textContent = "就绪";
    const first = await fetch(`../assets/examples/${EXAMPLES[0][0]}.lemma`);
    editor.value = await first.text();
    select.value = EXAMPLES[0][0];
    await runCheck();
  } catch (e) {
    output.innerHTML =
      `<span class="err">WASM 加载失败：${e.message}。请刷新重试。</span>`;
    status.textContent = "加载失败";
  }
})();
```

- [ ] **Step 2: 写 `play/index.html`（完整内容）**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Playground · Lemma</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist/index.css">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist-mono/index.css">
<link rel="stylesheet" href="../assets/theme.css">
</head>
<body>
<nav class="nav">
  <a class="brand" href="../"><span class="tri">▲</span>Lemma</a>
  <div class="links">
    <a href="../spec/">规格书</a>
    <a href="../white-paper/">白皮书</a>
    <a href="./">Playground</a>
    <a href="https://github.com/JunyuZhan/Lemma">GitHub</a>
  </div>
  <div class="spacer"></div>
  <span id="play-status" class="eyebrow" style="margin:0"></span>
</nav>
<main>
  <div class="play-layout">
    <div class="play-pane">
      <label for="example-select">示例</label>
      <select id="example-select" class="play-select"></select>
      <label for="editor">Lemma 源码</label>
      <textarea id="editor" class="play-editor" spellcheck="false"></textarea>
    </div>
    <div class="play-pane">
      <label>检查结果</label>
      <pre id="output" class="play-output">加载中…</pre>
    </div>
  </div>
</main>
<script type="module" src="../assets/play.js"></script>
</body>
</html>
```

- [ ] **Step 3: 本地起服做端到端冒烟**

Run:
```bash
cd /Users/apple/Documents/Project/Lemma-pages
python3 -m http.server 8642 >/tmp/lemma-http.log 2>&1 &
PID=$!
sleep 1
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8642/play/
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8642/assets/lemma.wasm
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8642/assets/examples/01-basics.lemma
kill $PID
```
Expected: 三个 200。浏览器人工验证（执行者截图或口述确认）：打开 `http://127.0.0.1:8642/play/`，右侧显示绿色「✓ 检查通过」；把编辑器中 `max` 改成 `m@x` 后约 300ms 出现红色语法错误；示例下拉切换生效。

- [ ] **Step 4: Commit**

```bash
git add assets/play.js play/index.html
git commit -m "feat: 在线 playground 页面（防抖检查 + 示例切换）"
```

---

### Task 5: 首页

**Files:**
- Create: `site/templates/home.html`
- Create: `site/templates/home.md`
- Modify: `site/build.sh`（文末追加首页段）

**Interfaces:**
- Consumes: `assets/theme.css` class 契约（Task 1）；`origin/main:examples/01-basics.lemma` 中 `fn max` 段（构建时提取）
- Produces: `index.html`（站点首页）；pandoc 变量 `$body$`、`$basecommit$`

- [ ] **Step 1: 写 `site/templates/home.md`（完整内容）**

```markdown
---
title: Lemma
---

<div class="hero">
<p class="eyebrow">面向 vibe coding 的编程语言</p>
<h1>AI 写实现，人和编译器共同验证意图。</h1>
<p class="sub">每个对外可见的函数必须携带 <code>spec</code>。<code>spec</code> 是唯一真相源——意图本身也是被验证的对象。</p>
<div class="cta">
<a class="btn btn-primary" href="play/">打开 Playground</a>
<a class="btn btn-ghost" href="spec/">阅读规格书</a>
</div>
</div>

<div class="section">
<p class="eyebrow">核心不变量</p>
<h2>三条规则，编译期强制</h2>
<div class="card-grid">
<div class="card">
<h3>可见函数必须有 spec</h3>
<p>每个对外可见的函数在定义处声明前置条件、后置条件与样例；没有 spec 的公开函数无法通过编译。</p>
</div>
<div class="card">
<h3>spec 是唯一真相源</h3>
<p>文档、注释、测试都不再各自为政——需求就写在代码旁边，并且可被执行。</p>
</div>
<div class="card">
<h3>spec 本身也被验证</h3>
<p>编译器同时检查：实现满足 spec，且 spec 非空洞、不矛盾、样例可达。</p>
</div>
</div>
</div>

<div class="section">
<p class="eyebrow">示例</p>
<h2>意图写进代码</h2>
<pre><code class="language-lemma">@@EXAMPLE@@</code></pre>
<p style="margin-top:16px"><a href="play/">在 Playground 中继续编辑 →</a></p>
</div>

<div class="section">
<p class="eyebrow">路线图</p>
<h2>当前进度：M2</h2>
<ul class="roadmap">
<li>M1 解析与类型检查</li>
<li class="current">M2 spec 自检</li>
<li>M3 运行时断言</li>
<li>M4 最小静态验证</li>
<li>M5 状态词汇</li>
</ul>
</div>
```

- [ ] **Step 2: 写 `site/templates/home.html`（完整 pandoc 模板）**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lemma — AI 写实现，共同验证意图</title>
<meta name="description" content="Lemma 是面向 vibe coding 的编程语言：每个可见函数必须携带 spec，编译器同时验证实现与 spec 本身。">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist/index.css">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource-variable/geist-mono/index.css">
<link rel="stylesheet" href="assets/theme.css">
</head>
<body>
<nav class="nav">
  <a class="brand" href="./"><span class="tri">▲</span>Lemma</a>
  <div class="links">
    <a href="spec/">规格书</a>
    <a href="white-paper/">白皮书</a>
    <a href="play/">Playground</a>
    <a href="https://github.com/JunyuZhan/Lemma">GitHub</a>
  </div>
</nav>
<main>
$body$
</main>
<footer class="footer">
  <a href="https://github.com/JunyuZhan/Lemma">GitHub 仓库</a>
  <span class="commit">基线 origin/main @ $basecommit$</span>
</footer>
</body>
</html>
```

- [ ] **Step 3: 在 `site/build.sh` 末尾追加首页段**

```bash

# 5) 首页（示例代码取自 origin/main 的 01-basics 中 fn max 段）
mkdir -p "$TMP"
awk '/^fn max/,/^}/' assets/examples/01-basics.lemma > "$TMP/example.lemma"
awk -v ex="$TMP/example.lemma" '
  /@@EXAMPLE@@/ { while ((getline line < ex) > 0) print line; close(ex); next }
  { print }
' site/templates/home.md > "$TMP/home.md"
pandoc "$TMP/home.md" --template site/templates/home.html \
  --metadata basecommit="$(git rev-parse --short origin/main)" \
  -o index.html
echo "首页构建完成: index.html"
```

- [ ] **Step 4: 运行构建并验证首页**

Run:
```bash
site/build.sh
grep -q 'AI 写实现，人和编译器共同验证意图' index.html
grep -q 'fn max' index.html
grep -q '@@EXAMPLE@@' index.html && echo "占位符残留!" || echo "占位符已替换"
grep -q '基线 origin/main @' index.html
grep -q 'play/' index.html
```
Expected: 除「占位符已替换」外各 grep 静默成功；最后一行输出「占位符已替换」

- [ ] **Step 5: 本地起服确认首页渲染**

Run:
```bash
python3 -m http.server 8642 >/tmp/lemma-http.log 2>&1 &
PID=$!
sleep 1
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8642/
kill $PID
```
Expected: `200`

- [ ] **Step 6: Commit**

```bash
git add site/templates/home.md site/templates/home.html site/build.sh
git commit -m "feat: 首页——核心价值、不变量三卡、示例与路线图"
```

---

### Task 6: 收尾——nojekyll、维护文档与全站验证

**Files:**
- Create: `.nojekyll`
- Create: `site/README.md`

**Interfaces:**
- Consumes: 前序全部任务产物
- Produces: 可 push 上线的完整站点；维护说明

- [ ] **Step 1: 写 `.nojekyll`（空文件）与 `site/README.md`**

`site/README.md` 完整内容：

```markdown
# Lemma 官网维护说明（gh-pages 分支）

站点内容**只从 `origin/main` 同步**；本分支不修改核心代码（`src/`、`spec.md`、
`docs/white-paper.md`、`examples/`、`conformance/`、`Cargo.toml`、`Cargo.lock`）。

## 构建

    site/build.sh

脚本会：校验 HEAD 不落后于 origin/main → `git show` 提取文档与示例 →
pandoc 渲染四页 → `cargo rustc --crate-type cdylib` 编译 WASM。

依赖：pandoc ≥ 3、rustup stable（含 wasm32-unknown-unknown target）、Node ≥ 20（测试）。

## 测试

    node --test site/tests/abi.test.mjs

## main 前进后的更新流程

    git fetch origin
    git rebase origin/main      # 网站文件与 main 无重叠，预期零冲突
    site/build.sh
    node --test site/tests/abi.test.mjs
    git add -A && git commit -m "chore: 同步 origin/main 并重建站点"
    git push origin gh-pages    # 必须显式分支名！分支跟踪了 origin/main

## 一致性红线

    git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock

必须为空。设计文档见 `docs/superpowers/specs/`。
```

Run:
```bash
touch .nojekyll
```

- [ ] **Step 2: 全量重建 + 测试 + 红线检查**

Run:
```bash
site/build.sh
node --test site/tests/abi.test.mjs
git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock
git status --short
```
Expected: 构建成功；4 tests PASS；红线 diff 为空；`git status` 只显示 `.nojekyll`、`site/README.md` 等预期新文件（`target/` 不出现——main 的 `.gitignore` 已含 `/target`）

- [ ] **Step 3: 四 URL 全通过验证**

Run:
```bash
python3 -m http.server 8642 >/tmp/lemma-http.log 2>&1 &
PID=$!
sleep 1
for u in / /spec/ /white-paper/ /play/; do
  printf "%s %s\n" "$u" "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:8642$u)"
done
kill $PID
```
Expected: 四行均为 `200`

- [ ] **Step 4: Commit**

```bash
git add .nojekyll site/README.md
git commit -m "chore: nojekyll 与站点维护说明"
```

- [ ] **Step 5: 向用户报告并等待上线指令（不要自行 push）**

向用户展示：`git log --oneline origin/main..HEAD` 的提交列表与四 URL 验证结果。明确告知下一步需用户确认后执行 `git push -u origin gh-pages`，然后在 GitHub Settings → Pages 选择 `gh-pages` / root，最后访问 `https://junuyzhan.github.io/Lemma/` 验证。
