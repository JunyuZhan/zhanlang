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
EXAMPLES="01-basics 02-records 03-io 04-batch 05-private-trusted 06-old"
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

# 4) WASM 构建（不改 Cargo.toml：命令行注入 cdylib）
export PATH="$(dirname "$(rustup which rustc)"):$(dirname "$(rustup which cargo)"):$PATH"
cargo rustc --release --lib --target wasm32-unknown-unknown -- --crate-type cdylib
cp target/wasm32-unknown-unknown/release/deps/lemma-*.wasm assets/lemma.wasm
echo "WASM 构建完成: assets/lemma.wasm"
