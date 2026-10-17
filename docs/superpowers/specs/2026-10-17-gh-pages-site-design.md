# Lemma 官网（gh-pages）设计 spec

日期：2026-10-17
状态：待审阅
基线：`origin/main` @ `7ecc431`（M2：spec 自检 + 库入口 + WASM 壳）

## 目标

为 Lemma 建立 GitHub Pages 官网：首页讲清核心价值，白皮书与规格书渲染为 HTML，在线 playground 用 WASM 实时验证 Lemma 源码。站点是纯展示层——核心语言代码、规格书源文、示例一律只从 `origin/main` 同步，本分支不修改它们。

非目标：

- 不做多语言（i18n）切换，站点文案与文档同为中文
- 不做 CI 自动部署（第一版手动 `site/build.sh` + push；Actions 留作后续）
- 不做深色模式
- 不实现 playground 的代码执行/单步——只做 `check`（词法/语法/类型/效果/spec 自检）结果展示，与 `lemma` CLI 的检查能力一致

## 分支与工作区

- worktree：`../Lemma-pages`（相对主仓库），分支 `gh-pages`，初始基点 `origin/main`
- `gh-pages` 分支**跟踪 `origin/main` 仅用于读取基准**；push 一律显式 `git push origin gh-pages`（禁止默认 push，防止误推 main）
- 分支保留完整源码树（`src/`、`Cargo.toml`、`examples/` 等与 `origin/main` 逐字节一致），网站文件叠加其上；这样 `cargo build --target wasm32-unknown-unknown` 可直接在分支内运行
- 一致性红线：`git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock` 必须为空；本 spec 文档位于 `docs/superpowers/specs/`（新目录，不在红线名单内）
- 分支根目录即 Pages 产物根，含 `.nojekyll`

## 信息架构

| 路径 | 内容 | 来源 |
|------|------|------|
| `/`（`index.html`） | 首页：核心价值、不变量、示例、CTA、路线图 | 手写模板 + `site/build.sh` 注入示例代码 |
| `/spec/` | 语言规格书 | `origin/main:spec.md` |
| `/white-paper/` | 白皮书 | `origin/main:docs/white-paper.md` |
| `/play/` | 在线 playground | `play/index.html` + `assets/lemma.wasm` + `assets/play.js` |

## 目录布局（gh-pages 分支）

```
（origin/main 原有文件全部保留）
index.html                  # 构建产物：首页
spec/index.html             # 构建产物：规格书
white-paper/index.html      # 构建产物：白皮书
play/index.html             # 手写：playground 页面
assets/theme.css            # Vercel 主题 tokens + 全站样式
assets/play.js              # WASM loader + playground 逻辑
assets/lemma.wasm           # 构建产物：cargo 编译输出
.nojekyll
site/build.sh               # 构建脚本
site/templates/             # pandoc 模板与首页片段
site/README.md              # 维护说明（更新流程）
docs/superpowers/specs/     # 本 spec 及后续设计文档
```

`index.html`、`spec/`、`white-paper/`、`assets/lemma.wasm` 由 `build.sh` 生成/覆盖，不手工编辑；`play/index.html`、`assets/theme.css`、`assets/play.js`、`site/` 为手写源文件。

## 构建流程（site/build.sh）

1. 基准校验：`git rev-parse HEAD` 与 `origin/main` 比对；若分支 HEAD 落后于 `origin/main`，打印警告并要求先 `git rebase origin/main`（不自动 rebase）
2. 取内容：一律 `git show origin/main:<path>` 提取 `spec.md`、`docs/white-paper.md`、`examples/*.lemma` 到临时目录，**禁止读工作区副本**（防止 rebase 前后混用）
3. 文档渲染：`pandoc --standalone --template site/templates/doc.html --toc --metadata title=...` 生成 `spec/index.html` 与 `white-paper/index.html`，输出注入 `assets/theme.css` 链接与顶栏导航
4. WASM 构建：`cargo build --release --target wasm32-unknown-unknown`，拷贝 `target/wasm32-unknown-unknown/release/lemma.wasm` → `assets/lemma.wasm`
5. 首页：`pandoc site/templates/home.md --template site/templates/home.html -o index.html`；示例代码块从 `origin/main:examples/01-basics.lemma` 截取带 spec 的函数，构建时注入 home.md 的代码围栏
6. 幂等：重跑不产生无意义 diff（固定 pandoc 版本行为、不内嵌时间戳）

WASM 构建命令（已在基线上验证：产物约 208KB，导出 `lemma_check` / `lemma_free` / `memory`）：

```bash
# 这台机器 PATH 里 Homebrew cargo/rustc 优先，rustup run 不覆盖 PATH，
# 必须显式把 rustup stable 工具链目录前置，否则 rustc 找不到 wasm32 的 std
export PATH="$(dirname "$(rustup which rustc)"):$(dirname "$(rustup which cargo)"):$PATH"
cargo rustc --release --lib --target wasm32-unknown-unknown -- --crate-type cdylib
# cdylib 产物带 hash，位于 deps/：拷贝唯一的 *.wasm 到 assets/lemma.wasm
cp target/wasm32-unknown-unknown/release/deps/lemma-*.wasm assets/lemma.wasm
```

说明：`Cargo.toml` 的 `[lib]` 未声明 `crate-type`，直接 `cargo build` 只产 rlib、不导出 C ABI 符号；用 `cargo rustc -- --crate-type cdylib` 在命令行注入，**不修改 Cargo.toml**（守一致性红线）。同时会产出 rlib，无害。

依赖：pandoc ≥ 3.x、Rust（rustup stable，需 `wasm32-unknown-unknown` target）、git。构建脚本用 `bash` + `set -euo pipefail`。

## 视觉风格（Vercel 参考，refero.styles）

落地 `assets/theme.css`，核心 tokens：

- 画布 `#fafafa`，卡片 `#ffffff`，发丝线 `#ebebeb`，主文本 `#171717`，正文 `#4d4d4d`，弱化 `#666666`
- 唯一彩色：Terminal Green `#297a3a`（playground 通过态、确认符 ✓）；链接也用它
- 字体：Geist Sans（正文/标题，weight 400/450/500，display 级 -0.06em 字距）+ Geist Mono（eyebrow 大写标签、代码、元数据，11-12px，0.071em 字距）；经 fontsource CDN 加载，失败 fallback `Inter / ui-sans-serif` 与 `JetBrains Mono / ui-monospace`
- 圆角：卡片/按钮 6px；深度只用发丝线（box-shadow ring），禁用投影
- 布局：max-width 1280px（文档页正文列 720px 居中，保证长文可读性），sticky 顶栏 64px + backdrop-blur，节间距 96-128px
- 组件：黑填充主按钮、发丝描边次按钮、eyebrow 大写标签、CLI 输出面板（`▲` 前缀命令、`✓` 绿色确认）

## Playground

- 页面：左侧编辑器（`<textarea>`，等宽字体，预填 `origin/main:examples/01-basics.lemma` 内容；下拉切换 5 个示例），右侧结果面板；窄屏上下堆叠
- 触发：输入防抖 300ms；示例切换立即检查
- WASM 调用（现有 C ABI，无 wasm-bindgen）：
  - **输入缓冲（JS 管理，零泄漏）**：初始化时对导出 `memory` 执行 `grow(1)` 得到 64KB scratch 页，位于原 `buffer.byteLength` 处；每次检查前把源码 `TextEncoder` 写入该区（若超出已 grow 容量则继续 grow），同一区域循环复用
  - `lemma_check(ptr, len) -> *mut u8`：返回 Rust 侧分配的 NUL 结尾 UTF-8；JS 用 `TextDecoder` 读取（按首 NUL 截断）
  - `lemma_free(ptr, len)`：释放时 `len` 为**含尾部 NUL 的字节数**（Rust 侧 `alloc` 时 push 了 0），loader 按 `找到的 NUL 下标 + 1` 传入
  - 每次检查后必须调用 `lemma_free` 归还结果字符串
  - 结果 `OK` → 面板绿 ✓「检查通过」；否则按行拆分展示错误
- 边界：非 UTF-8 输入由 Rust 侧兜底；WASM 加载失败时面板显示明确错误与刷新建议；不持久化用户编辑（无 localStorage，第一版）

## 首页内容

1. 顶栏：`▲ Lemma` 字标 + 规格书 / 白皮书 / Playground / GitHub 链接
2. Hero：主标题一句核心价值（取自 README 定位：AI 写实现，人和编译器共同验证意图，意图本身也是被验证的对象）；副标题一行；双 CTA「打开 Playground」「阅读规格书」
3. 核心不变量三卡：可见函数必须携带 spec；spec 是唯一真相源；编译器同时验证实现满足 spec 与 spec 本身非空洞/矛盾/不可达
4. 示例区：真实 Lemma 代码（带 spec 的函数），指向 /play/ 可继续编辑
5. 路线图条：M1→M5 简述，当前状态标注 M2
6. 页脚：GitHub 仓库链接、`origin/main` 基线 commit 短哈希（构建注入）。仓库当前无 LICENSE 文件，页脚不展示许可证行

## 错误处理与降级

- `origin/main` 含路径缺失时 `build.sh` 立即失败并指出路径
- CDN 字体失败：fallback 栈保证可读，无功能影响
- playground 的 wasm 404/编译失败：结果面板红字说明，编辑器仍可输入但不检查
- pandoc 渲染告警不视为失败；渲染错误（非零退出）视为构建失败

## 验证方式

- `site/build.sh` 全流程零错误，产物齐全
- `python3 -m http.server` 本地起服，四个 URL 均 200；`spec/`、`white-paper/` 页面含 TOC 与完整正文
- playground：合法示例显示 ✓；故意语法错误显示错误行；示例切换正常；连续快速输入无内存泄漏迹象（多次检查后 `lemma_free` 均被调用）
- `git diff origin/main -- src spec.md docs/white-paper.md examples conformance Cargo.toml Cargo.lock` 为空
- `git status` 无意外未跟踪文件（`target/`、临时目录已 gitignore 或排除）

## 上线（一次性人工步骤）

1. `git push -u origin gh-pages`（显式分支名）
2. GitHub 仓库 Settings → Pages → Source：Deploy from a branch → `gh-pages` / `/ (root)`
3. 访问 `https://junuyzhan.github.io/Lemma/` 验证（Pages 生效可能有分钟级延迟）

## 维护流程

main 前进后：worktree 内 `git fetch origin && git rebase origin/main` → 解决冲突（网站文件与 main 无重叠，预期零冲突）→ `site/build.sh` → `git add`（仅站点产物与手写源）→ commit → `git push origin gh-pages`。

## 风险

| 风险 | 应对 |
|------|------|
| 误 push 到 main | 分支 push 一律显式 refspec；build/维护文档写明；不设置 tracking push |
| rebase 后忘记重建导致内容过期 | `build.sh` 启动时校验 HEAD 与 `origin/main` 一致性并警告 |
| `lemma_free` 长度约定用错导致内存损坏 | loader 按「含 NUL」实现；验证章节含连续多次检查用例 |
| 字体 CDN 不可用 | fallback 栈；关键排版不依赖精确字距 |
