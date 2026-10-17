mod ast;
mod lexer;
mod parser;
mod typecheck;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: lemma <文件.lemma>");
        process::exit(2);
    }
    let path = &args[1];
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法读取 {}: {}", path, e);
            process::exit(1);
        }
    };
    let toks = match lexer::tokenize(&src) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("词法错误: {}", e);
            process::exit(1);
        }
    };
    if std::env::var("LEMMA_TRACE").as_deref() == Ok("tokens") {
        for t in &toks {
            eprintln!("{:?}", t);
        }
    }
    let module = match parser::parse(toks) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("语法错误: {}", e);
            process::exit(1);
        }
    };
    let errors = typecheck::Checker::new().check(&module);
    if errors.is_empty() {
        println!("OK: {} 通过解析与类型检查", path);
    } else {
        for e in errors {
            eprintln!("错误: {}", e);
        }
        process::exit(1);
    }
}
