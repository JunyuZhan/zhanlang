use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: lemma [run] <文件.lemma>");
        process::exit(2);
    }
    let mode = match args[1].as_str() {
        "run" => "run",
        "verify" => "verify",
        _ => "check",
    };
    let path = if mode == "check" { &args[1] } else { &args[2] };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法读取 {}: {}", path, e);
            process::exit(1);
        }
    };
    match mode {
        "check" => {
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
        "run" => {
            let out = lemma::run(&src);
            if out.starts_with("运行错误") || out.starts_with("类型检查未通过") {
                eprintln!("{}", out);
                process::exit(1);
            } else {
                println!("{}", out);
            }
        }
        _ => {
            println!("{}", lemma::verify(&src));
        }
    }
}
