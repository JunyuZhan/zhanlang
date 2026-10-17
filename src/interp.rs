use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Opt(Option<Box<Value>>),
    Res(Result<Box<Value>, Box<Value>>),
    Record(String, Vec<(String, Value)>),
    Variant(String, Vec<Value>),
    List(Vec<Value>),
    Unit,
}

pub fn run(module: &Module) -> Result<String, String> {
    Interp::new(module).run_examples()
}

struct Interp<'a> {
    module: &'a Module,
    fns: HashMap<&'a str, &'a FnDecl>,
    ctors: HashSet<&'a str>,
}

impl<'a> Interp<'a> {
    fn new(module: &'a Module) -> Self {
        let mut fns = HashMap::new();
        let mut ctors: HashSet<&str> = ["Some", "None", "Ok", "Err"].into();
        for d in &module.decls {
            match d {
                Decl::Fn(f) => {
                    fns.insert(f.name.as_str(), f);
                }
                Decl::Enum(e) => {
                    for v in &e.variants {
                        ctors.insert(v.name.as_str());
                    }
                }
                _ => {}
            }
        }
        Interp {
            module,
            fns,
            ctors,
        }
    }

    fn run_examples(&self) -> Result<String, String> {
        let mut passed = 0usize;
        let mut skipped = 0usize;
        let empty = HashMap::new();
        for d in &self.module.decls {
            if let Decl::Fn(f) = d {
                let Some(BodyPart::Spec(spec)) = &f.body_part else {
                    continue;
                };
                if body_has_todo(&f.body) {
                    skipped += spec.examples.len();
                    continue;
                }
                for ex in &spec.examples {
                    let Some((name, args)) = as_call(example_lhs(ex)) else {
                        continue;
                    };
                    if name != f.name {
                        continue;
                    }
                    let mut arg_vals = Vec::new();
                    for a in args {
                        arg_vals.push(self.eval(a, &empty)?);
                    }
                    let actual = self.call_fn(f, arg_vals)?;
                    match ex {
                        Example::Eq(_, rhs) => {
                            let expected = self.eval(rhs, &empty)?;
                            if actual != expected {
                                return Err(format!(
                                    "样例不符：`{}` 期望 {:?}，实际 {:?}",
                                    f.name, expected, actual
                                ));
                            }
                        }
                        Example::Is(_, pat) => {
                            if self.match_pattern(pat, &actual).is_none() {
                                return Err(format!(
                                    "样例不符：`{}` 的结果 {:?} 不匹配模式",
                                    f.name, actual
                                ));
                            }
                        }
                    }
                    passed += 1;
                }
            }
        }
        Ok(format!("{} 个样例通过，{} 个 todo 样例跳过", passed, skipped))
    }

    fn call_fn(&self, f: &FnDecl, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != f.params.len() {
            return Err(format!(
                "函数 `{}` 参数个数不匹配：期望 {}，实际 {}",
                f.name,
                f.params.len(),
                args.len()
            ));
        }
        let mut env: HashMap<String, Value> = f
            .params
            .iter()
            .zip(args)
            .map(|((n, _), v)| (n.clone(), v))
            .collect();

        if let Some(BodyPart::Spec(spec)) = &f.body_part {
            for p in &spec.pres {
                let v = self.eval(p, &env)?;
                if !truthy(&v)? {
                    return Err(format!("运行期 pre 违反：`{}`", f.name));
                }
            }
        }

        let result = self.exec_block(&f.body, &mut env)?;

        if let Some(BodyPart::Spec(spec)) = &f.body_part {
            env.insert("result".into(), result.clone());
            for p in &spec.posts {
                let v = self.eval(p, &env)?;
                if !truthy(&v)? {
                    return Err(format!("运行期 post 违反：`{}`", f.name));
                }
            }
        }
        Ok(result)
    }

    fn exec_block(&self, b: &Block, env: &mut HashMap<String, Value>) -> Result<Value, String> {
        for s in &b.stmts {
            match s {
                Stmt::Let { name, value, .. } => {
                    let v = self.eval(value, env)?;
                    env.insert(name.clone(), v);
                }
                Stmt::Return(e) => {
                    return match e {
                        Some(e) => self.eval(e, env),
                        None => Ok(Value::Unit),
                    };
                }
                Stmt::Expr(e) => {
                    self.eval(e, env)?;
                }
            }
        }
        match &b.tail {
            Some(e) => self.eval(e, env),
            None => Ok(Value::Unit),
        }
    }

    fn eval(&self, e: &Expr, env: &HashMap<String, Value>) -> Result<Value, String> {
        match e {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Todo => Err("todo 未实现".into()),
            Expr::Ident(name) => {
                if let Some(v) = env.get(name) {
                    return Ok(v.clone());
                }
                if name == "None" {
                    return Ok(Value::Opt(None));
                }
                if self.ctors.contains(name.as_str()) {
                    return Ok(Value::Variant(name.clone(), Vec::new()));
                }
                Err(format!("未定义的标识符 `{}`", name))
            }
            Expr::Unary(op, inner) => {
                let v = self.eval(inner, env)?;
                match (op, v) {
                    (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                    (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
                    _ => Err("一元运算类型错误".into()),
                }
            }
            Expr::Binary(op, l, r) => {
                let lv = self.eval(l, env)?;
                match op {
                    BinOp::And => {
                        if !truthy(&lv)? {
                            return Ok(Value::Bool(false));
                        }
                        let rv = self.eval(r, env)?;
                        Ok(Value::Bool(truthy(&rv)?))
                    }
                    BinOp::Or => {
                        if truthy(&lv)? {
                            return Ok(Value::Bool(true));
                        }
                        let rv = self.eval(r, env)?;
                        Ok(Value::Bool(truthy(&rv)?))
                    }
                    _ => {
                        let rv = self.eval(r, env)?;
                        eval_binary(*op, lv, rv)
                    }
                }
            }
            Expr::If(cond, then, els) => {
                let c = self.eval(cond, env)?;
                if truthy(&c)? {
                    self.exec_block(then, &mut env.clone())
                } else {
                    self.exec_block(els, &mut env.clone())
                }
            }
            Expr::Match(scrut, arms) => {
                let sv = self.eval(scrut, env)?;
                for (pat, arm) in arms {
                    if let Some(bindings) = self.match_pattern(pat, &sv) {
                        let mut arm_env = env.clone();
                        arm_env.extend(bindings);
                        return self.eval(arm, &arm_env);
                    }
                }
                Err("match 无匹配分支".into())
            }
            Expr::Is(lhs, pat) => {
                let v = self.eval(lhs, env)?;
                Ok(Value::Bool(self.match_pattern(pat, &v).is_some()))
            }
            Expr::Field(base, name) => {
                let bv = self.eval(base, env)?;
                match bv {
                    Value::Record(_, fields) => fields
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, v)| v.clone())
                        .ok_or_else(|| format!("记录没有字段 `{}`", name)),
                    Value::Str(s) if name == "len" => Ok(Value::Int(s.chars().count() as i64)),
                    other => Err(format!("无法对 {:?} 取字段 `{}`", other, name)),
                }
            }
            Expr::Index(base, idx) => {
                let bv = self.eval(base, env)?;
                let iv = self.eval(idx, env)?;
                match (bv, iv) {
                    (Value::List(items), Value::Int(i)) => items
                        .get(i as usize)
                        .cloned()
                        .ok_or_else(|| "索引越界".to_string()),
                    _ => Err("索引类型错误".into()),
                }
            }
            Expr::RecordLit(name, fields) => {
                let mut vals = Vec::new();
                for (n, fe) in fields {
                    vals.push((n.clone(), self.eval(fe, env)?));
                }
                Ok(Value::Record(name.clone(), vals))
            }
            Expr::List(items) => {
                let mut vals = Vec::new();
                for it in items {
                    vals.push(self.eval(it, env)?);
                }
                Ok(Value::List(vals))
            }
            Expr::Call(callee, args) => {
                if let Expr::Field(base, method) = &**callee {
                    let bv = self.eval(base, env)?;
                    let mut av = Vec::new();
                    for a in args {
                        av.push(self.eval(a, env)?);
                    }
                    return method_call(bv, method, av);
                }
                if let Expr::Ident(name) = &**callee {
                    let mut av = Vec::new();
                    for a in args {
                        av.push(self.eval(a, env)?);
                    }
                    return self.call_ident(name, av);
                }
                Err("不可调用的表达式".into())
            }
        }
    }

    fn call_ident(&self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match name {
            "None" => Ok(Value::Opt(None)),
            "Some" => Ok(Value::Opt(Some(Box::new(args.into_iter().next().ok_or("Some 需要一个参数")?)))),
            "Ok" => Ok(Value::Res(Ok(Box::new(args.into_iter().next().ok_or("Ok 需要一个参数")?)))),
            "Err" => Ok(Value::Res(Err(Box::new(args.into_iter().next().ok_or("Err 需要一个参数")?)))),
            _ => {
                if self.ctors.contains(name) {
                    return Ok(Value::Variant(name.to_string(), args));
                }
                if let Some(f) = self.fns.get(name) {
                    return self.call_fn(f, args);
                }
                Err(format!("未定义的函数或构造子 `{}`", name))
            }
        }
    }

    fn match_pattern(&self, pat: &Pattern, v: &Value) -> Option<HashMap<String, Value>> {
        match pat {
            Pattern::Wildcard => Some(HashMap::new()),
            Pattern::Bind(name) => {
                let mut m = HashMap::new();
                m.insert(name.clone(), v.clone());
                Some(m)
            }
            Pattern::Lit(_) => None,
            Pattern::Ctor(name, subs) => match (name.as_str(), v) {
                ("None", Value::Opt(None)) => Some(HashMap::new()),
                ("Some", Value::Opt(Some(inner))) => self.match_pattern(&subs[0], inner),
                ("Ok", Value::Res(Ok(val))) => self.match_pattern(&subs[0], val),
                ("Err", Value::Res(Err(e))) => self.match_pattern(&subs[0], e),
                (n, Value::Variant(vn, fields)) if n == vn => {
                    let mut bindings = HashMap::new();
                    for (s, fv) in subs.iter().zip(fields) {
                        bindings.extend(self.match_pattern(s, fv)?);
                    }
                    Some(bindings)
                }
                _ => None,
            },
        }
    }
}

fn as_call(e: &Expr) -> Option<(&str, &[Expr])> {
    if let Expr::Call(callee, args) = e {
        if let Expr::Ident(name) = &**callee {
            return Some((name.as_str(), args.as_slice()));
        }
    }
    None
}

fn example_lhs(ex: &Example) -> &Expr {
    match ex {
        Example::Eq(l, _) => l,
        Example::Is(l, _) => l,
    }
}

fn body_has_todo(b: &Block) -> bool {
    for s in &b.stmts {
        if stmt_has_todo(s) {
            return true;
        }
    }
    b.tail.as_ref().map(expr_has_todo).unwrap_or(false)
}

fn stmt_has_todo(s: &Stmt) -> bool {
    match s {
        Stmt::Let { value, .. } => expr_has_todo(value),
        Stmt::Return(e) => e.as_ref().map(expr_has_todo).unwrap_or(false),
        Stmt::Expr(e) => expr_has_todo(e),
    }
}

fn expr_has_todo(e: &Expr) -> bool {
    if matches!(e, Expr::Todo) {
        return true;
    }
    match e {
        Expr::Call(c, args) => expr_has_todo(c) || args.iter().any(expr_has_todo),
        Expr::Field(b, _) | Expr::Unary(_, b) => expr_has_todo(b),
        Expr::Index(a, b) | Expr::Binary(_, a, b) => expr_has_todo(a) || expr_has_todo(b),
        Expr::RecordLit(_, fs) => fs.iter().any(|(_, v)| expr_has_todo(v)),
        Expr::List(items) => items.iter().any(expr_has_todo),
        Expr::If(c, a, b) => expr_has_todo(c) || block_has_todo(a) || block_has_todo(b),
        Expr::Match(s, arms) => {
            expr_has_todo(s) || arms.iter().any(|(_, e)| expr_has_todo(e))
        }
        Expr::Is(v, _) => expr_has_todo(v),
        _ => false,
    }
}

fn block_has_todo(b: &Block) -> bool {
    b.stmts.iter().any(stmt_has_todo)
        || b.tail.as_ref().map(expr_has_todo).unwrap_or(false)
}

fn truthy(v: &Value) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        other => Err(format!("期望 Bool，实际 {:?}", other)),
    }
}

fn eval_binary(op: BinOp, lv: Value, rv: Value) -> Result<Value, String> {
    use BinOp::*;
    match op {
        Eq => Ok(Value::Bool(lv == rv)),
        Ne => Ok(Value::Bool(lv != rv)),
        Lt | Le | Gt | Ge | Add | Sub | Mul | Div | Mod => {
            if let (Value::Int(a), Value::Int(b)) = (&lv, &rv) {
                let (a, b) = (*a, *b);
                return Ok(match op {
                    Lt => Value::Bool(a < b),
                    Le => Value::Bool(a <= b),
                    Gt => Value::Bool(a > b),
                    Ge => Value::Bool(a >= b),
                    Add => Value::Int(a + b),
                    Sub => Value::Int(a - b),
                    Mul => Value::Int(a * b),
                    Div => {
                        if b == 0 {
                            return Err("除以零".into());
                        }
                        Value::Int(a / b)
                    }
                    Mod => {
                        if b == 0 {
                            return Err("除以零".into());
                        }
                        Value::Int(a % b)
                    }
                    _ => unreachable!(),
                });
            }
            if let (Value::Float(a), Value::Float(b)) = (&lv, &rv) {
                let (a, b) = (*a, *b);
                return Ok(match op {
                    Lt => Value::Bool(a < b),
                    Le => Value::Bool(a <= b),
                    Gt => Value::Bool(a > b),
                    Ge => Value::Bool(a >= b),
                    Add => Value::Float(a + b),
                    Sub => Value::Float(a - b),
                    Mul => Value::Float(a * b),
                    Div => Value::Float(a / b),
                    Mod => Value::Float(a % b),
                    _ => unreachable!(),
                });
            }
            Err("算术运算类型错误".into())
        }
        And | Or => Err("逻辑运算应短路处理".into()),
    }
}

fn method_call(bv: Value, method: &str, args: Vec<Value>) -> Result<Value, String> {
    match (bv, method) {
        (Value::Str(s), "len") => Ok(Value::Int(s.chars().count() as i64)),
        (Value::Str(s), "contains") => match args.first() {
            Some(Value::Str(sub)) => Ok(Value::Bool(s.contains(sub))),
            _ => Err("contains 需要一个字符串".into()),
        },
        (Value::Str(s), "starts_with") => match args.first() {
            Some(Value::Str(sub)) => Ok(Value::Bool(s.starts_with(sub))),
            _ => Err("starts_with 需要一个字符串".into()),
        },
        (Value::Str(s), "ends_with") => match args.first() {
            Some(Value::Str(sub)) => Ok(Value::Bool(s.ends_with(sub))),
            _ => Err("ends_with 需要一个字符串".into()),
        },
        (Value::List(items), "len") => Ok(Value::Int(items.len() as i64)),
        (Value::List(items), "is_empty") => Ok(Value::Bool(items.is_empty())),
        (Value::List(items), "get") => match args.first() {
            Some(Value::Int(i)) => Ok(Value::Opt(
                items
                    .get(*i as usize)
                    .cloned()
                    .map(Box::new),
            )),
            _ => Err("get 需要一个整数".into()),
        },
        (Value::Int(n), "to_float") => Ok(Value::Float(n as f64)),
        (Value::Float(f), "to_int") => Ok(Value::Int(f as i64)),
        (other, m) => Err(format!("{:?} 没有方法 `{}`", other, m)),
    }
}
