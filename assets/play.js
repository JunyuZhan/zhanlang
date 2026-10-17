import { loadZl } from "./zl-abi.mjs";

const ZH = {
  ok: "✓ 检查通过",
  errors: "有错误",
  loading: "加载 WASM…",
  ready: "就绪",
  failed: "加载失败",
  loadFailed: "WASM 加载失败",
  checkFailed: "检查失败",
};
const EN = {
  ok: "✓ Check passed",
  errors: "Errors",
  loading: "Loading WASM…",
  ready: "Ready",
  failed: "Load failed",
  loadFailed: "WASM load failed",
  checkFailed: "Check failed",
};

const L = document.documentElement.lang.startsWith("zh") ? ZH : EN;

const EXAMPLES = [
  ["01-basics", "Basics: pure functions & spec", "基础：纯函数与 spec"],
  ["02-records", "Records & enums", "记录与枚举"],
  ["03-io", "IO & effects", "IO 与效果"],
  ["04-batch", "Concurrency", "并发"],
  ["05-private-trusted", "Private & trusted", "私有与 trusted"],
  ["06-old", "old semantics", "old 语义"],
];

const editor = document.getElementById("editor");
const output = document.getElementById("output");
const select = document.getElementById("example-select");
const status = document.getElementById("play-status");

const langIndex = L === ZH ? 2 : 1;
for (const row of EXAMPLES) {
  const opt = document.createElement("option");
  opt.value = row[0];
  opt.textContent = row[langIndex];
  select.appendChild(opt);
}

function render(text) {
  if (text === "OK") {
    output.innerHTML = `<span class="ok">${L.ok}</span>`;
    status.textContent = "OK";
    return;
  }
  const escaped = text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  output.innerHTML = `<span class="err">${escaped}</span>`;
  status.textContent = L.errors;
}

const base = new URL(".", import.meta.url);
let mod = null;
let timer = null;

async function runCheck() {
  if (!mod) return;
  try {
    render(mod.check(editor.value));
  } catch (e) {
    output.innerHTML = `<span class="err">${L.checkFailed}: ${e.message}</span>`;
    status.textContent = L.failed;
  }
}

function scheduleCheck() {
  clearTimeout(timer);
  timer = setTimeout(runCheck, 300);
}

editor.addEventListener("input", scheduleCheck);

select.addEventListener("change", async () => {
  const resp = await fetch(new URL(`./examples/${select.value}.zl`, base));
  editor.value = await resp.text();
  runCheck();
});

(async () => {
  status.textContent = L.loading;
  try {
    mod = await loadZl(new URL("./zl.wasm", base).href);
    status.textContent = L.ready;
    const first = await fetch(new URL(`./examples/${EXAMPLES[0][0]}.zl`, base));
    editor.value = await first.text();
    select.value = EXAMPLES[0][0];
    await runCheck();
  } catch (e) {
    output.innerHTML = `<span class="err">${L.loadFailed}: ${e.message}。请刷新重试。</span>`;
    status.textContent = L.failed;
  }
})();
