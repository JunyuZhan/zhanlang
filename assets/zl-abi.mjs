// zhanlang WASM ABI 封装：浏览器与 Node 通用。
// ABI 约定（与 src/lib.rs wasm 壳一致）：
//   zl_check(ptr, len) -> NUL 结尾 UTF-8 指针（Rust 侧分配）
//   zl_free(ptr, len)  -> len 为含尾部 NUL 的字节数
// 输入缓冲：JS 管理的 64KB scratch 页循环复用，零泄漏。

export async function loadZl(source) {
  let bytes;
  if (typeof source === "string") {
    const resp = await fetch(source);
    if (!resp.ok) throw new Error(`加载 WASM 失败: HTTP ${resp.status}`);
    bytes = await resp.arrayBuffer();
  } else if (source instanceof ArrayBuffer) {
    bytes = source;
  } else {
    bytes = source.buffer.slice(
      source.byteOffset,
      source.byteOffset + source.byteLength
    );
  }

  const { instance } = await WebAssembly.instantiate(bytes, {});
  const { zl_check, zl_free, memory } = instance.exports;
  if (typeof zl_check !== "function" || typeof zl_free !== "function") {
    throw new Error("WASM 缺少 zl_check/zl_free 导出");
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
      const outPtr = zl_check(ptr, input.length);
      const view = new Uint8Array(memory.buffer);
      let end = outPtr;
      while (view[end] !== 0) end++;
      const text = decoder.decode(view.subarray(outPtr, end));
      zl_free(outPtr, end - outPtr + 1);
      return text;
    },
  };
}
