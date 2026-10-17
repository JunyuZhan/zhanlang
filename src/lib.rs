mod ast;
mod lexer;
mod parser;
mod typecheck;

/// 稳定入口：解析并检查一段 Lemma 源码。
///
/// 这是库对外的唯一稳定 API。命令行二进制、WebAssembly playground
/// 以及未来的 LSP 都通过它调用，不接触内部结构。
///
/// 返回 "OK" 表示通过；否则每行一条错误信息。
pub fn check(source: &str) -> String {
    match lexer::tokenize(source) {
        Err(e) => format!("词法错误: {}", e),
        Ok(toks) => match parser::parse(toks) {
            Err(e) => format!("语法错误: {}", e),
            Ok(module) => {
                let errors = typecheck::Checker::new().check(&module);
                if errors.is_empty() {
                    "OK".to_string()
                } else {
                    errors.join("\n")
                }
            }
        },
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    fn alloc(s: String) -> *mut u8 {
        let mut bytes = s.into_bytes();
        bytes.push(0);
        let boxed = bytes.into_boxed_slice();
        let ptr = boxed.as_ptr() as *mut u8;
        std::mem::forget(boxed);
        ptr
    }

    /// 检查 Lemma 源码；返回 NUL 结尾的 UTF-8 结果，须用 `lemma_free` 释放。
    #[no_mangle]
    pub extern "C" fn lemma_check(ptr: *const u8, len: usize) -> *mut u8 {
        let src = if ptr.is_null() {
            String::new()
        } else {
            let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
            match std::str::from_utf8(bytes) {
                Ok(s) => s.to_string(),
                Err(_) => return alloc("输入不是合法 UTF-8".to_string()),
            }
        };
        alloc(super::check(&src))
    }

    #[no_mangle]
    pub extern "C" fn lemma_free(ptr: *mut u8, len: usize) {
        if !ptr.is_null() && len > 0 {
            unsafe {
                let slice = std::slice::from_raw_parts_mut(ptr, len);
                drop(Box::from_raw(slice as *mut [u8]));
            }
        }
    }
}
