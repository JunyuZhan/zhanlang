import { loadLemma } from "./lemma-abi.mjs";

const EXAMPLES = [
  ["01-basics", "Basics: pure functions & spec"],
  ["02-records", "Records & enums"],
  ["03-io", "IO & effects"],
  ["04-batch", "Concurrency"],
  ["05-private-trusted", "Private & trusted"],
  ["06-old", "old semantics"],
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
    output.innerHTML = '<span class="ok">✓ Check passed</span>';
    status.textContent = "OK";
    return;
  }
  const escaped = text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  output.innerHTML = `<span class="err">${escaped}</span>`;
  status.textContent = "Errors";
}

let mod = null;
let timer = null;

async function runCheck() {
  if (!mod) return;
  try {
    render(mod.check(editor.value));
  } catch (e) {
    output.innerHTML = `<span class="err">Check failed: ${e.message}</span>`;
    status.textContent = "Failed";
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
  status.textContent = "Loading WASM…";
  try {
    mod = await loadLemma("../assets/lemma.wasm");
    status.textContent = "Ready";
    const first = await fetch(`../assets/examples/${EXAMPLES[0][0]}.lemma`);
    editor.value = await first.text();
    select.value = EXAMPLES[0][0];
    await runCheck();
  } catch (e) {
    output.innerHTML = `<span class="err">WASM load failed: ${e.message}. Please refresh to retry.</span>`;
    status.textContent = "Load failed";
  }
})();
