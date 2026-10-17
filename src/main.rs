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
    let out = lemma::check(&src);
    if out == "OK" {
        println!("OK: {} 通过解析与类型检查", path);
    } else {
        for line in out.lines() {
            eprintln!("错误: {}", line);
        }
        process::exit(1);
    }
}
