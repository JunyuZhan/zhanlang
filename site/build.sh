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
git show origin/main:spec.zh-CN.md > "$TMP/spec.zh.md"
git show origin/main:docs/white-paper.zh-CN.md > "$TMP/white-paper.zh.md"
mkdir -p assets/examples
EXAMPLES="01-basics 02-records 03-io 04-batch 05-private-trusted 06-old"
for name in $EXAMPLES; do
  git show "origin/main:examples/$name.lemma" > "assets/examples/$name.lemma"
done

# 3) 文档渲染（EN 默认 + ZH）
mkdir -p spec white-paper zh/spec zh/white-paper
pandoc "$TMP/spec.md" --standalone --template site/templates/doc.html \
  --toc --metadata title="Language specification" --metadata lang=en \
  --metadata root="../" --metadata switch-url="../zh/spec/" --metadata switch-label="中文" \
  -o spec/index.html
pandoc "$TMP/white-paper.md" --standalone --template site/templates/doc.html \
  --toc --metadata title="Whitepaper" --metadata lang=en \
  --metadata root="../" --metadata switch-url="../zh/white-paper/" --metadata switch-label="中文" \
  -o white-paper/index.html
pandoc "$TMP/spec.zh.md" --standalone --template site/templates/doc.html \
  --toc --metadata title="语言规格书" --metadata lang=zh-CN \
  --metadata root="../../" --metadata switch-url="../../spec/" --metadata switch-label="English" \
  -o zh/spec/index.html
pandoc "$TMP/white-paper.zh.md" --standalone --template site/templates/doc.html \
  --toc --metadata title="白皮书" --metadata lang=zh-CN \
  --metadata root="../../" --metadata switch-url="../../white-paper/" --metadata switch-label="English" \
  -o zh/white-paper/index.html

echo "文档构建完成: spec/ white-paper/ zh/spec/ zh/white-paper/"

# 4) WASM 构建（不改 Cargo.toml：命令行注入 cdylib）
export PATH="$(dirname "$(rustup which rustc)"):$(dirname "$(rustup which cargo)"):$PATH"
cargo rustc --release --lib --target wasm32-unknown-unknown -- --crate-type cdylib
cp target/wasm32-unknown-unknown/release/deps/lemma-*.wasm assets/lemma.wasm
echo "WASM 构建完成: assets/lemma.wasm"

# 5) 首页（示例代码取自 origin/main 的 01-basics 中 fn max 段）
mkdir -p "$TMP" zh
awk '/^fn max/,/^}/' assets/examples/01-basics.lemma > "$TMP/example.lemma"
awk -v ex="$TMP/example.lemma" '
  /@@EXAMPLE@@/ { while ((getline line < ex) > 0) print line; close(ex); next }
  { print }
' site/templates/home.md > "$TMP/home.md"
awk -v ex="$TMP/example.lemma" '
  /@@EXAMPLE@@/ { while ((getline line < ex) > 0) print line; close(ex); next }
  { print }
' site/templates/home.zh-CN.md > "$TMP/home.zh.md"
pandoc "$TMP/home.md" --template site/templates/home.html \
  --metadata basecommit="$(git rev-parse --short origin/main)" \
  --metadata lang=en --metadata root="./" \
  --metadata switch-url="zh/" --metadata switch-label="中文" \
  -o index.html
pandoc "$TMP/home.zh.md" --template site/templates/home.html \
  --metadata basecommit="$(git rev-parse --short origin/main)" \
  --metadata lang=zh-CN --metadata root="../" \
  --metadata switch-url="../" --metadata switch-label="English" \
  -o zh/index.html
echo "首页构建完成: index.html zh/index.html"
