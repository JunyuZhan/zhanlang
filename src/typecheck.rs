use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Named(String, Vec<Ty>),
    Opt(Box<Ty>),
    Unknown,
}

impl Ty {
    fn from_ast(t: &Type) -> Ty {
        match t {
            Type::Int => Ty::Int,
            Type::Float => Ty::Float,
            Type::Bool => Ty::Bool,
            Type::Str => Ty::Str,
            Type::Record(_) => Ty::Unknown,
            Type::Named(n, args) => Ty::Named(n.clone(), args.iter().map(Ty::from_ast).collect()),
            Type::Opt(inner) => Ty::Opt(Box::new(Ty::from_ast(inner))),
        }
    }
}

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<(String, Ty)>,
    ret: Option<Ty>,
    effects: Vec<Effect>,
}

enum TypeKind {
    Alias(Ty),
    Record(Vec<(String, Ty)>),
    Enum,
}

struct TypeInfo {
    kind: TypeKind,
}

#[derive(Clone, Copy, PartialEq)]
enum SpecCtx {
    Pre,
    Post,
    Example,
}

type Env = HashMap<String, Ty>;

pub struct Checker {
    types: HashMap<String, TypeInfo>,
    fns: HashMap<String, FnSig>,
    ctors: HashSet<String>,
    errors: Vec<String>,
}

impl Checker {
    pub fn new() -> Self {
        let mut ctors = HashSet::new();
        for c in ["Some", "None", "Ok", "Err"] {
            ctors.insert(c.to_string());
        }
        Checker {
            types: HashMap::new(),
            fns: HashMap::new(),
            ctors,
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, module: &Module) -> Vec<String> {
        self.collect(module);
        for decl in &module.decls {
            if let Decl::Fn(f) = decl {
                self.check_fn(f);
            }
        }
        self.errors.clone()
    }

    fn collect(&mut self, module: &Module) {
        for decl in &module.decls {
            match decl {
                Decl::Type(t) => {
                    let kind = match &t.ty {
                        Type::Record(fields) => TypeKind::Record(
                            fields
                                .iter()
                                .map(|(n, ty)| (n.clone(), Ty::from_ast(ty)))
                                .collect(),
                        ),
                        other => TypeKind::Alias(Ty::from_ast(other)),
                    };
                    self.types.insert(
                        t.name.clone(),
                        TypeInfo { kind },
                    );
                }
                Decl::Enum(e) => {
                    for v in &e.variants {
                        self.ctors.insert(v.name.clone());
                    }
                    self.types.insert(
                        e.name.clone(),
                        TypeInfo {
                            kind: TypeKind::Enum,
                        },
                    );
                }
                Decl::Fn(f) => {
                    let params = f
                        .params
                        .iter()
                        .map(|(n, t)| (n.clone(), Ty::from_ast(t)))
                        .collect();
                    let ret = f.ret.as_ref().map(Ty::from_ast);
                    self.fns.insert(
                        f.name.clone(),
                        FnSig {
                            params,
                            ret,
                            effects: f.effects.clone(),
                        },
                    );
                }
            }
        }
    }

    fn record_fields(&self, name: &str) -> Option<Vec<(String, Ty)>> {
        match self.types.get(name) {
            Some(TypeInfo {
                kind: TypeKind::Record(f),
            }) => Some(f.clone()),
            Some(TypeInfo {
                kind: TypeKind::Alias(Ty::Named(n, _)),
            }) => self.record_fields(n),
            _ => None,
        }
    }

    fn check_fn(&mut self, f: &FnDecl) {
        let mut env: Env = f
            .params
            .iter()
            .map(|(n, t)| (n.clone(), Ty::from_ast(t)))
            .collect();
        let ret = f.ret.as_ref().map(Ty::from_ast);

        match &f.body_part {
            None => {
                if !f.private {
                    self.err(format!("公开函数 `{}` 缺少 spec", f.name));
                } else {
                    self.err(format!(
                        "私有函数 `{}` 缺少 spec 时必须标注 trusted(\"原因\")",
                        f.name
                    ));
                }
            }
            Some(BodyPart::Trusted(reason)) => {
                if !f.private {
                    self.err(format!(
                        "`trusted(\"{}\")` 只允许用在 private 函数上，但 `{}` 是公开的",
                        reason, f.name
                    ));
                }
            }
            Some(BodyPart::Spec(spec)) => {
                self.check_spec(f, spec, &mut env, ret.as_ref());
            }
        }

        let body_ty = self.check_block(&f.body, &mut env, &f.effects, None, ret.as_ref());
        if let Some(ret_ty) = ret.as_ref() {
            if let Some(bty) = &body_ty {
                if !self.compat(bty, ret_ty) {
                    self.err(format!(
                        "函数 `{}` 的返回类型不匹配：期望 {:?}，实际 {:?}",
                        f.name, ret_ty, bty
                    ));
                }
            }
        }
    }

    fn check_spec(
        &mut self,
        f: &FnDecl,
        spec: &SpecBlock,
        env: &mut Env,
        ret: Option<&Ty>,
    ) {
        for p in &spec.pres {
            self.check_expr(p, env, &f.effects, Some(SpecCtx::Pre), ret);
        }
        for p in &spec.posts {
            if matches!(p, Expr::Bool(true)) {
                self.err(format!("函数 `{}` 的 post 恒真，违反非空洞性", f.name));
            }
            self.check_expr(p, env, &f.effects, Some(SpecCtx::Post), ret);
        }
        for ex in &spec.examples {
            match ex {
                Example::Eq(lhs, rhs) => {
                    self.check_expr(lhs, env, &f.effects, Some(SpecCtx::Example), ret);
                    self.check_expr(rhs, env, &f.effects, Some(SpecCtx::Example), ret);
                }
                Example::Is(lhs, pat) => {
                    self.check_expr(lhs, env, &f.effects, Some(SpecCtx::Example), ret);
                    self.check_pattern(pat, env);
                }
            }
        }
        self.check_reachability(f, spec);
        self.check_example_consistency(f, spec);
    }

    fn check_reachability(&mut self, f: &FnDecl, spec: &SpecBlock) {
        for p in &spec.posts {
            if let Expr::Match(_, arms) = p {
                for (_, arm) in arms {
                    if let Expr::Binary(op, lhs, rhs) = arm {
                        if let Expr::Ident(pname) = &**lhs {
                            if let Expr::Int(c1) = &**rhs {
                                let c1 = *c1;
                                for pre in &spec.pres {
                                    if let Expr::Binary(op2, l2, r2) = pre {
                                        if let Expr::Ident(pn2) = &**l2 {
                                            if pn2 == pname {
                                                if let Expr::Int(c2) = &**r2 {
                                                    if contradicts(*op, c1, *op2, *c2) {
                                                        self.err(format!(
                                                            "函数 `{}` 的 post 分支与 pre 矛盾，不可达",
                                                            f.name
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_example_consistency(&mut self, f: &FnDecl, spec: &SpecBlock) {
        let empty = HashMap::new();
        for ex in &spec.examples {
            match ex {
                Example::Eq(lhs, rhs) => {
                    if let Some((name, args)) = as_call(lhs) {
                        if name == f.name {
                            let vals: Option<Vec<Value>> =
                                args.iter().map(|a| eval(a, &empty)).collect();
                            if let Some(vals) = vals {
                                let mut env = param_env(&f.params, &vals);
                                for p in &spec.pres {
                                    if let Some(Value::Bool(false)) = eval(p, &env) {
                                        self.err(format!("函数 `{}` 的样例输入不满足 pre", f.name));
                                    }
                                }
                                if let Some(rv) = eval(rhs, &empty) {
                                    env.insert("result".into(), rv);
                                    for p in &spec.posts {
                                        if let Some(Value::Bool(false)) = eval(p, &env) {
                                            self.err(format!(
                                                "函数 `{}` 的样例输出与 post 矛盾",
                                                f.name
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Example::Is(lhs, _) => {
                    if let Some((name, args)) = as_call(lhs) {
                        if name == f.name {
                            let vals: Option<Vec<Value>> =
                                args.iter().map(|a| eval(a, &empty)).collect();
                            if let Some(vals) = vals {
                                let env = param_env(&f.params, &vals);
                                for p in &spec.pres {
                                    if let Some(Value::Bool(false)) = eval(p, &env) {
                                        self.err(format!("函数 `{}` 的样例输入不满足 pre", f.name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_block(
        &mut self,
        b: &Block,
        env: &mut Env,
        effects: &[Effect],
        spec_ctx: Option<SpecCtx>,
        ret: Option<&Ty>,
    ) -> Option<Ty> {
        for s in &b.stmts {
            match s {
                Stmt::Let { name, ty, value } => {
                    let vt = self.check_expr(value, env, effects, spec_ctx, ret);
                    let declared = ty.as_ref().map(Ty::from_ast);
                    if let Some(d) = &declared {
                        if !self.compat(&vt, d) {
                            self.err(format!(
                                "变量 `{}` 类型不匹配：期望 {:?}，实际 {:?}",
                                name, d, vt
                            ));
                        }
                    }
                    let final_ty = declared.unwrap_or(vt);
                    env.insert(name.clone(), final_ty);
                }
                Stmt::Return(e) => {
                    if let Some(v) = e {
                        self.check_expr(v, env, effects, spec_ctx, ret);
                    }
                }
                Stmt::Expr(e) => {
                    self.check_expr(e, env, effects, spec_ctx, ret);
                }
            }
        }
        b.tail
            .as_ref()
            .map(|t| self.check_expr(t, env, effects, spec_ctx, ret))
    }

    fn check_pattern(&mut self, p: &Pattern, env: &Env) {
        match p {
            Pattern::Wildcard => {}
            Pattern::Lit(_) => {}
            Pattern::Bind(_) => {}
            Pattern::Ctor(name, subs) => {
                if name != "Some"
                    && name != "None"
                    && name != "Ok"
                    && name != "Err"
                    && !self.ctors.contains(name)
                {
                    self.err(format!("未定义的构造子 `{}`", name));
                }
                for s in subs {
                    self.check_pattern(s, env);
                }
            }
        }
    }

    fn bind_pattern(&self, pat: &Pattern, env: &mut Env) {
        match pat {
            Pattern::Wildcard | Pattern::Lit(_) => {}
            Pattern::Bind(name) => {
                env.insert(name.clone(), Ty::Unknown);
            }
            Pattern::Ctor(_, subs) => {
                for s in subs {
                    self.bind_pattern(s, env);
                }
            }
        }
    }

    fn check_expr(
        &mut self,
        e: &Expr,
        env: &mut Env,
        effects: &[Effect],
        spec_ctx: Option<SpecCtx>,
        ret: Option<&Ty>,
    ) -> Ty {
        match e {
            Expr::Int(_) => Ty::Int,
            Expr::Float(_) => Ty::Float,
            Expr::Bool(_) => Ty::Bool,
            Expr::Str(_) => Ty::Str,
            Expr::Todo => Ty::Unknown,
            Expr::Ident(name) => {
                if let Some(t) = env.get(name) {
                    return t.clone();
                }
                if spec_ctx == Some(SpecCtx::Post) && name == "result" {
                    return ret.cloned().unwrap_or(Ty::Unknown);
                }
                if name == "events" {
                    if spec_ctx == Some(SpecCtx::Post) {
                        return Ty::Named("events".into(), Vec::new());
                    }
                    self.err("`events` 只能出现在 post 中".into());
                    return Ty::Unknown;
                }
                if self.ctors.contains(name) {
                    return Ty::Unknown;
                }
                self.err(format!("未定义的标识符 `{}`", name));
                Ty::Unknown
            }
            Expr::Call(callee, args) => self.check_call(callee, args, env, effects, spec_ctx, ret),
            Expr::Field(base, name) => {
                let bt = self.check_expr(base, env, effects, spec_ctx, ret);
                if let Ty::Opt(_) = bt {
                    self.err(format!("对 `T?` 直接字段访问 `{}`，须先解包", name));
                    return Ty::Unknown;
                }
                let builtin = match &bt {
                    Ty::Str => self.builtin_method("String", name),
                    Ty::Int => self.builtin_method("Int", name),
                    Ty::Float => self.builtin_method("Float", name),
                    Ty::Named(n, _) => self.builtin_method(n, name),
                    _ => None,
                };
                if let Some(ft) = builtin {
                    return ft;
                }
                if let Ty::Named(n, _) = &bt {
                    if let Some(fields) = self.record_fields(n) {
                        if let Some((_, ft)) = fields.iter().find(|(fname, _)| fname == name) {
                            return ft.clone();
                        }
                    }
                    self.err(format!("类型 `{}` 没有字段 `{}`", n, name));
                    return Ty::Unknown;
                }
                self.err(format!("无法对 {:?} 取字段 `{}`", bt, name));
                Ty::Unknown
            }
            Expr::Index(base, idx) => {
                let bt = self.check_expr(base, env, effects, spec_ctx, ret);
                self.check_expr(idx, env, effects, spec_ctx, ret);
                match bt {
                    Ty::Named(n, args) if n == "List" => args.first().cloned().unwrap_or(Ty::Unknown),
                    Ty::Unknown => Ty::Unknown,
                    other => {
                        self.err(format!("无法对 {:?} 做索引", other));
                        Ty::Unknown
                    }
                }
            }
            Expr::RecordLit(name, fields) => {
                let declared: Option<Vec<(String, Ty)>> = self.record_fields(name);
                for (fname, fexpr) in fields {
                    let ft = self.check_expr(fexpr, env, effects, spec_ctx, ret);
                    if let Some(dfields) = &declared {
                        match dfields.iter().find(|(n, _)| n == fname) {
                            Some((_, dty)) => {
                                if !self.compat(&ft, dty) {
                                    self.err(format!(
                                        "记录字段 `{}` 类型不匹配：期望 {:?}，实际 {:?}",
                                        fname, dty, ft
                                    ));
                                }
                            }
                            None => self.err(format!("记录 `{}` 没有字段 `{}`", name, fname)),
                        }
                    }
                }
                Ty::Named(name.clone(), Vec::new())
            }
            Expr::List(items) => {
                let mut elem = Ty::Unknown;
                for it in items {
                    let t = self.check_expr(it, env, effects, spec_ctx, ret);
                    if matches!(elem, Ty::Unknown) {
                        elem = t;
                    }
                }
                Ty::Named("List".into(), vec![elem])
            }
            Expr::If(cond, then, els) => {
                let ct = self.check_expr(cond, env, effects, spec_ctx, ret);
                if !matches!(ct, Ty::Bool) && !matches!(ct, Ty::Unknown) {
                    self.err("if 条件必须是 Bool".into());
                }
                let tt = self.check_block(then, env, effects, spec_ctx, ret);
                let et = self.check_block(els, env, effects, spec_ctx, ret);
                match (tt, et) {
                    (Some(a), Some(b)) if a != b && !matches!(a, Ty::Unknown) && !matches!(b, Ty::Unknown) => {
                        self.err(format!("if 分支类型不一致：{:?} vs {:?}", a, b));
                        Ty::Unknown
                    }
                    (a, b) => a.or(b).unwrap_or(Ty::Unknown),
                }
            }
            Expr::Match(scrut, arms) => {
                self.check_expr(scrut, env, effects, spec_ctx, ret);
                let mut result = Ty::Unknown;
                for (pat, arm) in arms {
                    self.check_pattern(pat, env);
                    let mut arm_env = env.clone();
                    self.bind_pattern(pat, &mut arm_env);
                    let at = self.check_expr(arm, &mut arm_env, effects, spec_ctx, ret);
                    result = if matches!(result, Ty::Unknown) {
                        at
                    } else if result == at {
                        result
                    } else {
                        Ty::Unknown
                    };
                }
                result
            }
            Expr::Is(lhs, pat) => {
                self.check_expr(lhs, env, effects, spec_ctx, ret);
                self.check_pattern(pat, env);
                Ty::Bool
            }
            Expr::Unary(op, inner) => {
                let it = self.check_expr(inner, env, effects, spec_ctx, ret);
                match op {
                    UnOp::Not => {
                        if !matches!(it, Ty::Bool) && !matches!(it, Ty::Unknown) {
                            self.err("! 只能作用于 Bool".into());
                        }
                        Ty::Bool
                    }
                    UnOp::Neg => it,
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                let lt = self.check_expr(lhs, env, effects, spec_ctx, ret);
                let rt = self.check_expr(rhs, env, effects, spec_ctx, ret);
                match op {
                    BinOp::And | BinOp::Or => Ty::Bool,
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Ty::Bool,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if lt != rt && !matches!(lt, Ty::Unknown) && !matches!(rt, Ty::Unknown) {
                            self.err(format!("二元运算左右类型不一致：{:?} vs {:?}", lt, rt));
                        }
                        lt
                    }
                }
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        env: &mut Env,
        effects: &[Effect],
        spec_ctx: Option<SpecCtx>,
        ret: Option<&Ty>,
    ) -> Ty {
        let arg_tys: Vec<Ty> = args
            .iter()
            .map(|a| self.check_expr(a, env, effects, spec_ctx, ret))
            .collect();
        if let Expr::Ident(name) = callee {
            if name == "old" {
                if spec_ctx != Some(SpecCtx::Post) {
                    self.err("`old` 只能出现在 post 中".into());
                }
                return Ty::Unknown;
            }
            if let Some(sig) = self.fns.get(name).cloned() {
                if sig.params.len() != args.len() {
                    self.err(format!(
                        "函数 `{}` 参数个数不匹配：期望 {}，实际 {}",
                        name,
                        sig.params.len(),
                        args.len()
                    ));
                } else {
                    for (i, at) in arg_tys.iter().enumerate() {
                        if !self.compat(&at, &sig.params[i].1) {
                            self.err(format!(
                                "函数 `{}` 第 {} 个参数类型不匹配：期望 {:?}，实际 {:?}",
                                name,
                                i + 1,
                                sig.params[i].1,
                                at
                            ));
                        }
                    }
                }
                for eff in &sig.effects {
                    if !effects.contains(eff) {
                        self.err(format!(
                            "纯函数（或效果不足）调用了声明 `{:?}` 的函数 `{}`",
                            eff, name
                        ));
                    }
                }
                return sig.ret.unwrap_or(Ty::Unknown);
            }
            if self.ctors.contains(name) {
                return self.ctor_ty(name, &arg_tys);
            }
            self.err(format!("未定义的函数或构造子 `{}`", name));
            return Ty::Unknown;
        }
        self.check_expr(callee, env, effects, spec_ctx, ret);
        Ty::Unknown
    }

    fn ctor_ty(&self, name: &str, arg_tys: &[Ty]) -> Ty {
        match name {
            "None" => Ty::Opt(Box::new(Ty::Unknown)),
            "Some" => Ty::Opt(Box::new(arg_tys.first().cloned().unwrap_or(Ty::Unknown))),
            "Ok" => Ty::Named(
                "Result".into(),
                vec![arg_tys.first().cloned().unwrap_or(Ty::Unknown), Ty::Unknown],
            ),
            "Err" => Ty::Named(
                "Result".into(),
                vec![Ty::Unknown, arg_tys.first().cloned().unwrap_or(Ty::Unknown)],
            ),
            _ => {
                if let Some(TypeInfo {
                    kind: TypeKind::Enum,
                }) = self.types.get(name)
                {
                    Ty::Named(name.to_string(), Vec::new())
                } else {
                    Ty::Unknown
                }
            }
        }
    }

    fn builtin_method(&self, base: &str, method: &str) -> Option<Ty> {
        match (base, method) {
            ("String", "len") => Some(Ty::Int),
            ("String", "contains") => Some(Ty::Bool),
            ("String", "starts_with") => Some(Ty::Bool),
            ("String", "ends_with") => Some(Ty::Bool),
            ("Int", "to_float") => Some(Ty::Float),
            ("Float", "to_int") => Some(Ty::Int),
            ("events", "len") => Some(Ty::Int),
            ("events", "contains") => Some(Ty::Bool),
            _ => None,
        }
    }

    fn compat(&self, a: &Ty, b: &Ty) -> bool {
        let ra = self.resolve(a);
        let rb = self.resolve(b);
        matches!(&ra, Ty::Unknown)
            || matches!(&rb, Ty::Unknown)
            || ra == rb
            || match (&ra, &rb) {
                (Ty::Named(x, _), Ty::Named(y, _)) => x == y,
                _ => false,
            }
    }

    fn resolve(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Named(n, _) => match self.types.get(n) {
                Some(TypeInfo {
                    kind: TypeKind::Alias(target),
                }) => self.resolve(target),
                _ => ty.clone(),
            },
            Ty::Opt(inner) => Ty::Opt(Box::new(self.resolve(inner))),
            _ => ty.clone(),
        }
    }

    fn err(&mut self, msg: String) {
        self.errors.push(msg);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

fn as_call(e: &Expr) -> Option<(&str, &[Expr])> {
    if let Expr::Call(callee, args) = e {
        if let Expr::Ident(name) = &**callee {
            return Some((name.as_str(), args.as_slice()));
        }
    }
    None
}

fn param_env(params: &[(String, Type)], vals: &[Value]) -> HashMap<String, Value> {
    params
        .iter()
        .zip(vals)
        .map(|((n, _), v)| (n.clone(), v.clone()))
        .collect()
}

fn eval(e: &Expr, env: &HashMap<String, Value>) -> Option<Value> {
    match e {
        Expr::Int(n) => Some(Value::Int(*n)),
        Expr::Float(f) => Some(Value::Float(*f)),
        Expr::Bool(b) => Some(Value::Bool(*b)),
        Expr::Str(s) => Some(Value::Str(s.clone())),
        Expr::Ident(name) => env.get(name).cloned(),
        Expr::Unary(op, inner) => {
            let v = eval(inner, env)?;
            match (op, v) {
                (UnOp::Not, Value::Bool(b)) => Some(Value::Bool(!b)),
                (UnOp::Neg, Value::Int(n)) => Some(Value::Int(-n)),
                (UnOp::Neg, Value::Float(f)) => Some(Value::Float(-f)),
                _ => None,
            }
        }
        Expr::Binary(op, l, r) => {
            let lv = eval(l, env)?;
            let rv = eval(r, env)?;
            eval_binary(*op, lv, rv)
        }
        Expr::Call(callee, args) => {
            if let Expr::Field(base, method) = &**callee {
                let bv = eval(base, env)?;
                let arg_vals: Option<Vec<Value>> = args.iter().map(|a| eval(a, env)).collect();
                return eval_method(bv, method, arg_vals?);
            }
            None
        }
        _ => None,
    }
}

fn eval_binary(op: BinOp, lv: Value, rv: Value) -> Option<Value> {
    use BinOp::*;
    match op {
        Eq => Some(Value::Bool(lv == rv)),
        Ne => Some(Value::Bool(lv != rv)),
        And => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a && b)),
            _ => None,
        },
        Or => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a || b)),
            _ => None,
        },
        Lt | Le | Gt | Ge | Add | Sub | Mul | Div | Mod => {
            if let (Value::Int(a), Value::Int(b)) = (&lv, &rv) {
                let (a, b) = (*a, *b);
                return match op {
                    Lt => Some(Value::Bool(a < b)),
                    Le => Some(Value::Bool(a <= b)),
                    Gt => Some(Value::Bool(a > b)),
                    Ge => Some(Value::Bool(a >= b)),
                    Add => Some(Value::Int(a + b)),
                    Sub => Some(Value::Int(a - b)),
                    Mul => Some(Value::Int(a * b)),
                    Div => {
                        if b == 0 {
                            None
                        } else {
                            Some(Value::Int(a / b))
                        }
                    }
                    Mod => {
                        if b == 0 {
                            None
                        } else {
                            Some(Value::Int(a % b))
                        }
                    }
                    _ => unreachable!(),
                };
            }
            if let (Value::Float(a), Value::Float(b)) = (&lv, &rv) {
                let (a, b) = (*a, *b);
                return match op {
                    Lt => Some(Value::Bool(a < b)),
                    Le => Some(Value::Bool(a <= b)),
                    Gt => Some(Value::Bool(a > b)),
                    Ge => Some(Value::Bool(a >= b)),
                    Add => Some(Value::Float(a + b)),
                    Sub => Some(Value::Float(a - b)),
                    Mul => Some(Value::Float(a * b)),
                    Div => Some(Value::Float(a / b)),
                    Mod => Some(Value::Float(a % b)),
                    _ => unreachable!(),
                };
            }
            None
        }
    }
}

fn eval_method(bv: Value, method: &str, args: Vec<Value>) -> Option<Value> {
    match (bv, method) {
        (Value::Str(s), "len") => Some(Value::Int(s.chars().count() as i64)),
        (Value::Str(s), "contains") => match args.first() {
            Some(Value::Str(sub)) => Some(Value::Bool(s.contains(sub))),
            _ => None,
        },
        (Value::Str(s), "starts_with") => match args.first() {
            Some(Value::Str(sub)) => Some(Value::Bool(s.starts_with(sub))),
            _ => None,
        },
        (Value::Str(s), "ends_with") => match args.first() {
            Some(Value::Str(sub)) => Some(Value::Bool(s.ends_with(sub))),
            _ => None,
        },
        (Value::Int(n), "to_float") => Some(Value::Float(n as f64)),
        (Value::Float(f), "to_int") => Some(Value::Int(f as i64)),
        _ => None,
    }
}

fn contradicts(op1: BinOp, c1: i64, op2: BinOp, c2: i64) -> bool {
    use BinOp::*;
    match (op1, op2) {
        (Le, Gt) => c2 >= c1,
        (Lt, Ge) => c2 >= c1,
        (Ge, Lt) => c1 >= c2,
        (Gt, Le) => c1 >= c2,
        (Eq, Ne) => c1 == c2,
        (Ne, Eq) => c1 == c2,
        _ => false,
    }
}
