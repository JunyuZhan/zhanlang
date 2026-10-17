use std::path::Path;

fn zl_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) == Some("zl") {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn examples_pass() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    for p in zl_files(&dir) {
        let src = std::fs::read_to_string(&p).unwrap();
        assert_eq!(zhanlang::check(&src), "OK", "示例应通过: {:?}", p);
    }
}

#[test]
fn reject_cases_fail() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/reject");
    for p in zl_files(&dir) {
        let src = std::fs::read_to_string(&p).unwrap();
        assert_ne!(zhanlang::check(&src), "OK", "拒绝用例应报错: {:?}", p);
    }
}

#[test]
fn run_pure_examples() {
    for name in ["01-basics", "02-records", "05-private-trusted", "06-old"] {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(format!("{}.zl", name));
        let src = std::fs::read_to_string(p).unwrap();
        let out = zhanlang::run(&src);
        assert!(
            out.contains("样例通过"),
            "{} 应可运行，实际: {}",
            name,
            out
        );
    }
}

#[test]
fn verify_max() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("01-basics.zl");
    let src = std::fs::read_to_string(p).unwrap();
    let out = zhanlang::verify(&src);
    assert!(out.contains("max: verified"), "max 应被静态证明，实际:\n{}", out);
}
