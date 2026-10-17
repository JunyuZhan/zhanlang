use crate::ast::*;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq)]
enum Sym {
    C(i64),
    V(String),
    Add(Box<Sym>, Box<Sym>),
    Sub(Box<Sym>, Box<Sym>),
    Mul(Box<Sym>, Box<Sym>),
    Div(Box<Sym>, Box<Sym>),
    Neg(Box<Sym>),
    Ite(Box<SB>, Box<Sym>, Box<Sym>),
    Opaque,
}

#[derive(Clone, Debug, PartialEq)]
enum SB {
    T,
    F,
    Cmp(CmpOp, Box<Sym>, Box<Sym>),
    And(Box<SB>, Box<SB>),
    Or(Box<SB>, Box<SB>),
    Not(Box<SB>),
    Opaque,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Clone, Debug, PartialEq)]
enum SVal {
    Int(Sym),
    Bool(SB),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Proved,
    Disproved,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Lin {
    c: i64,
    t: BTreeMap<String, i64>,
}

pub fn verify(module: &Module) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for d in &module.decls {
        if let Decl::Fn(f) = d {
            out.push((f.name.clone(), verify_fn(f)));
        }
    }
    out
}

fn verify_fn(f: &FnDecl) -> bool {
    let Some(BodyPart::Spec(spec)) = &f.body_part else {
        return false;
    };
    let mut env: HashMap<String, SVal> = f
        .params
        .iter()
        .map(|(n, _)| (n.clone(), SVal::Int(Sym::V(n.clone()))))
        .collect();
    let body = sym_block(&f.body, &mut env);
    let result_sym = match body {
        SVal::Int(s) => s,
        _ => return false,
    };

    let mut assumptions: Vec<SB> = Vec::new();
    for p in &spec.pres {
        if let SVal::Bool(b) = sym_eval(p, &env) {
            assumptions.push(b);
        } else {
            return false;
        }
    }

    for post in &spec.posts {
        let mut post_env = env.clone();
        post_env.insert("result".into(), SVal::Int(result_sym.clone()));
        let goal = match sym_eval(post, &post_env) {
            SVal::Bool(b) => b,
            _ => return false,
        };
        let v = prove(&assumptions, &goal);
        if v != Verdict::Proved {
            return false;
        }
    }
    true
}

fn sym_block(b: &Block, env: &mut HashMap<String, SVal>) -> SVal {
    for s in &b.stmts {
        match s {
            Stmt::Let { name, value, .. } => {
                let v = sym_eval(value, env);
                env.insert(name.clone(), v);
            }
            Stmt::Return(e) => {
                return match e {
                    Some(e) => sym_eval(e, env),
                    None => SVal::Int(Sym::Opaque),
                };
            }
            Stmt::Expr(e) => {
                sym_eval(e, env);
            }
        }
    }
    match &b.tail {
        Some(e) => sym_eval(e, env),
        None => SVal::Int(Sym::Opaque),
    }
}

fn sym_eval(e: &Expr, env: &HashMap<String, SVal>) -> SVal {
    match e {
        Expr::Int(n) => SVal::Int(Sym::C(*n)),
        Expr::Bool(b) => SVal::Bool(if *b { SB::T } else { SB::F }),
        Expr::Ident(name) => env.get(name).cloned().unwrap_or(SVal::Int(Sym::Opaque)),
        Expr::Unary(op, inner) => match (op, sym_eval(inner, env)) {
            (UnOp::Neg, SVal::Int(s)) => SVal::Int(Sym::Neg(Box::new(s))),
            (UnOp::Not, SVal::Bool(s)) => SVal::Bool(SB::Not(Box::new(s))),
            _ => SVal::Int(Sym::Opaque),
        },
        Expr::Binary(op, l, r) => {
            let lv = sym_eval(l, env);
            let rv = sym_eval(r, env);
            match (op, lv, rv) {
                (BinOp::Add, SVal::Int(a), SVal::Int(b)) => {
                    SVal::Int(Sym::Add(Box::new(a), Box::new(b)))
                }
                (BinOp::Sub, SVal::Int(a), SVal::Int(b)) => {
                    SVal::Int(Sym::Sub(Box::new(a), Box::new(b)))
                }
                (BinOp::Mul, SVal::Int(a), SVal::Int(b)) => {
                    SVal::Int(Sym::Mul(Box::new(a), Box::new(b)))
                }
                (BinOp::Div, SVal::Int(a), SVal::Int(b)) => {
                    SVal::Int(Sym::Div(Box::new(a), Box::new(b)))
                }
                (BinOp::And, SVal::Bool(a), SVal::Bool(b)) => {
                    SVal::Bool(SB::And(Box::new(a), Box::new(b)))
                }
                (BinOp::Or, SVal::Bool(a), SVal::Bool(b)) => {
                    SVal::Bool(SB::Or(Box::new(a), Box::new(b)))
                }
                (op, SVal::Int(a), SVal::Int(b)) => SVal::Bool(SB::Cmp(
                    cmp_of(op),
                    Box::new(a),
                    Box::new(b),
                )),
                _ => SVal::Int(Sym::Opaque),
            }
        }
        Expr::If(cond, then, els) => {
            let c = match sym_eval(cond, env) {
                SVal::Bool(b) => b,
                _ => SB::Opaque,
            };
            let a = sym_block(then, &mut env.clone());
            let b = sym_block(els, &mut env.clone());
            match (a, b) {
                (SVal::Int(x), SVal::Int(y)) => {
                    SVal::Int(Sym::Ite(Box::new(c), Box::new(x), Box::new(y)))
                }
                _ => SVal::Int(Sym::Opaque),
            }
        }
        _ => SVal::Int(Sym::Opaque),
    }
}

fn cmp_of(op: &BinOp) -> CmpOp {
    match op {
        BinOp::Eq => CmpOp::Eq,
        BinOp::Ne => CmpOp::Ne,
        BinOp::Lt => CmpOp::Lt,
        BinOp::Le => CmpOp::Le,
        BinOp::Gt => CmpOp::Gt,
        BinOp::Ge => CmpOp::Ge,
        _ => CmpOp::Eq,
    }
}

fn prove(assumptions: &[SB], goal: &SB) -> Verdict {
    if let Some((node, cond, a, b)) = find_ite_in_sb(goal) {
        let g1 = subst_ite_in_sb(goal, &node, &a);
        let g2 = subst_ite_in_sb(goal, &node, &b);
        let mut ass1 = assumptions.to_vec();
        ass1.push(cond.clone());
        let mut ass2 = assumptions.to_vec();
        ass2.push(SB::Not(Box::new(cond)));
        return and_verdict(prove(&ass1, &g1), prove(&ass2, &g2));
    }
    match goal {
        SB::T => Verdict::Proved,
        SB::F | SB::Opaque => Verdict::Unknown,
        SB::And(a, b) => and_verdict(prove(assumptions, a), prove(assumptions, b)),
        SB::Or(a, b) => or_verdict(prove(assumptions, a), prove(assumptions, b)),
        SB::Not(a) => not_verdict(prove(assumptions, a)),
        SB::Cmp(op, l, r) => prove_cmp(assumptions, *op, l, r),
    }
}

fn prove_cmp(assumptions: &[SB], op: CmpOp, l: &Sym, r: &Sym) -> Verdict {
    if l == r {
        return match op {
            CmpOp::Eq | CmpOp::Ge | CmpOp::Le => Verdict::Proved,
            CmpOp::Ne | CmpOp::Gt | CmpOp::Lt => Verdict::Disproved,
        };
    }
    let lin = match (linearize(l), linearize(r)) {
        (Some(a), Some(b)) => sub_lin(&a, &b),
        _ => return Verdict::Unknown,
    };
    if lin.t.is_empty() {
        return decide_const(op, lin.c);
    }
    for a in assumptions {
        if let Some((aop, alin)) = atomic_of(a) {
            if alin == lin {
                return implies_same(op, aop);
            }
            if neg_lin(&alin) == lin {
                return implies_same(op, negate_op(aop));
            }
        }
    }
    Verdict::Unknown
}

fn decide_const(op: CmpOp, c: i64) -> Verdict {
    let b = match op {
        CmpOp::Lt => c < 0,
        CmpOp::Le => c <= 0,
        CmpOp::Gt => c > 0,
        CmpOp::Ge => c >= 0,
        CmpOp::Eq => c == 0,
        CmpOp::Ne => c != 0,
    };
    if b {
        Verdict::Proved
    } else {
        Verdict::Disproved
    }
}

fn implies_same(goal: CmpOp, ass: CmpOp) -> Verdict {
    use CmpOp::*;
    let b = match goal {
        Ge => matches!(ass, Ge | Gt | Eq),
        Gt => matches!(ass, Gt),
        Le => matches!(ass, Le | Lt | Eq),
        Lt => matches!(ass, Lt),
        Eq => matches!(ass, Eq),
        Ne => matches!(ass, Ne),
    };
    if b {
        Verdict::Proved
    } else {
        let d = match goal {
            Ge => matches!(ass, Lt),
            Gt => matches!(ass, Le | Lt | Eq),
            Le => matches!(ass, Gt),
            Lt => matches!(ass, Ge | Gt | Eq),
            Eq => matches!(ass, Ne | Lt | Gt),
            Ne => matches!(ass, Eq),
        };
        if d {
            Verdict::Disproved
        } else {
            Verdict::Unknown
        }
    }
}

fn negate_op(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        CmpOp::Eq => CmpOp::Eq,
        CmpOp::Ne => CmpOp::Ne,
    }
}

fn atomic_of(sb: &SB) -> Option<(CmpOp, Lin)> {
    match sb {
        SB::Cmp(op, l, r) => {
            let lin = sub_lin(&linearize(l)?, &linearize(r)?);
            Some((*op, lin))
        }
        SB::Not(inner) => {
            if let SB::Cmp(op, l, r) = &**inner {
                let lin = sub_lin(&linearize(l)?, &linearize(r)?);
                Some((negate_op(*op), lin))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn linearize(s: &Sym) -> Option<Lin> {
    match s {
        Sym::C(n) => Some(Lin {
            c: *n,
            t: BTreeMap::new(),
        }),
        Sym::V(n) => {
            let mut t = BTreeMap::new();
            t.insert(n.clone(), 1);
            Some(Lin { c: 0, t })
        }
        Sym::Add(a, b) => Some(add_lin(linearize(a)?, linearize(b)?)),
        Sym::Sub(a, b) => Some(sub_lin(&linearize(a)?, &linearize(b)?)),
        Sym::Neg(a) => Some(neg_lin(&linearize(a)?)),
        Sym::Mul(a, b) => {
            let la = linearize(a)?;
            let lb = linearize(b)?;
            if la.t.is_empty() {
                Some(scale_lin(&lb, la.c))
            } else if lb.t.is_empty() {
                Some(scale_lin(&la, lb.c))
            } else {
                None
            }
        }
        Sym::Div(_, _) => None,
        _ => None,
    }
}

fn add_lin(a: Lin, b: Lin) -> Lin {
    let mut t = a.t.clone();
    for (k, v) in b.t {
        *t.entry(k).or_insert(0) += v;
    }
    t.retain(|_, v| *v != 0);
    Lin { c: a.c + b.c, t }
}

fn sub_lin(a: &Lin, b: &Lin) -> Lin {
    let mut t = a.t.clone();
    for (k, v) in &b.t {
        *t.entry(k.clone()).or_insert(0) -= v;
    }
    t.retain(|_, v| *v != 0);
    Lin { c: a.c - b.c, t }
}

fn neg_lin(a: &Lin) -> Lin {
    Lin {
        c: -a.c,
        t: a.t.iter().map(|(k, v)| (k.clone(), -v)).collect(),
    }
}

fn scale_lin(a: &Lin, k: i64) -> Lin {
    Lin {
        c: a.c * k,
        t: a.t.iter().map(|(n, v)| (n.clone(), v * k)).collect(),
    }
}

fn find_ite(s: &Sym) -> Option<(Sym, SB, Sym, Sym)> {
    match s {
        Sym::Ite(c, a, b) => Some((s.clone(), (**c).clone(), (**a).clone(), (**b).clone())),
        Sym::Add(a, b) | Sym::Sub(a, b) | Sym::Mul(a, b) | Sym::Div(a, b) => {
            find_ite(a).or_else(|| find_ite(b))
        }
        Sym::Neg(a) => find_ite(a),
        _ => None,
    }
}

fn find_ite_in_sb(sb: &SB) -> Option<(Sym, SB, Sym, Sym)> {
    match sb {
        SB::Cmp(_, l, r) => find_ite(l).or_else(|| find_ite(r)),
        SB::And(a, b) | SB::Or(a, b) => find_ite_in_sb(a).or_else(|| find_ite_in_sb(b)),
        SB::Not(a) => find_ite_in_sb(a),
        _ => None,
    }
}

fn subst_ite(s: &Sym, node: &Sym, repl: &Sym) -> Sym {
    if s == node {
        return repl.clone();
    }
    match s {
        Sym::Add(a, b) => Sym::Add(Box::new(subst_ite(a, node, repl)), Box::new(subst_ite(b, node, repl))),
        Sym::Sub(a, b) => Sym::Sub(Box::new(subst_ite(a, node, repl)), Box::new(subst_ite(b, node, repl))),
        Sym::Mul(a, b) => Sym::Mul(Box::new(subst_ite(a, node, repl)), Box::new(subst_ite(b, node, repl))),
        Sym::Div(a, b) => Sym::Div(Box::new(subst_ite(a, node, repl)), Box::new(subst_ite(b, node, repl))),
        Sym::Neg(a) => Sym::Neg(Box::new(subst_ite(a, node, repl))),
        Sym::Ite(c, a, b) => Sym::Ite(
            c.clone(),
            Box::new(subst_ite(a, node, repl)),
            Box::new(subst_ite(b, node, repl)),
        ),
        _ => s.clone(),
    }
}

fn subst_ite_in_sb(sb: &SB, node: &Sym, repl: &Sym) -> SB {
    match sb {
        SB::Cmp(op, l, r) => SB::Cmp(
            *op,
            Box::new(subst_ite(l, node, repl)),
            Box::new(subst_ite(r, node, repl)),
        ),
        SB::And(a, b) => SB::And(
            Box::new(subst_ite_in_sb(a, node, repl)),
            Box::new(subst_ite_in_sb(b, node, repl)),
        ),
        SB::Or(a, b) => SB::Or(
            Box::new(subst_ite_in_sb(a, node, repl)),
            Box::new(subst_ite_in_sb(b, node, repl)),
        ),
        SB::Not(a) => SB::Not(Box::new(subst_ite_in_sb(a, node, repl))),
        _ => sb.clone(),
    }
}

fn and_verdict(a: Verdict, b: Verdict) -> Verdict {
    use Verdict::*;
    match (a, b) {
        (Proved, Proved) => Proved,
        (Disproved, _) | (_, Disproved) => Disproved,
        _ => Unknown,
    }
}

fn or_verdict(a: Verdict, b: Verdict) -> Verdict {
    use Verdict::*;
    match (a, b) {
        (Proved, _) | (_, Proved) => Proved,
        (Disproved, Disproved) => Disproved,
        _ => Unknown,
    }
}

fn not_verdict(v: Verdict) -> Verdict {
    match v {
        Verdict::Proved => Verdict::Disproved,
        Verdict::Disproved => Verdict::Proved,
        Verdict::Unknown => Verdict::Unknown,
    }
}
