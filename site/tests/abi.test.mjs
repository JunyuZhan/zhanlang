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
