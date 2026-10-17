use crate::ast::*;
use crate::lexer::Tok;

pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    no_record: bool,
}

pub fn parse(toks: Vec<Tok>) -> Result<Module, String> {
    let mut p = Parser {
        toks,
        pos: 0,
        no_record: false,
    };
    p.module()
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }
    fn bump(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, t: Tok) -> bool {
        if self.peek() == &t {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: Tok) -> Result<(), String> {
        if self.eat(t.clone()) {
            Ok(())
        } else {
            Err(format!("期望 {:?}，实际 {:?}", t, self.peek()))
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            other => Err(format!("期望标识符，实际 {:?}", other)),
        }
    }

    fn module(&mut self) -> Result<Module, String> {
        let mut name = None;
        if self.eat(Tok::Module) {
            name = Some(self.ident()?);
        }
        let mut decls = Vec::new();
        while *self.peek() != Tok::Eof {
            decls.push(self.decl()?);
        }
        Ok(Module { name, decls })
    }

    fn decl(&mut self) -> Result<Decl, String> {
        if self.eat(Tok::Type) {
            let name = self.ident()?;
            self.expect(Tok::Assign)?;
            let ty = self.ty()?;
            return Ok(Decl::Type(TypeDecl { name, ty }));
        }
        if self.eat(Tok::Enum) {
            let name = self.ident()?;
            self.expect(Tok::LBrace)?;
            let mut variants = Vec::new();
            while !self.eat(Tok::RBrace) {
                let vname = self.ident()?;
                let mut fields = Vec::new();
                if self.eat(Tok::LParen) {
                    while !self.eat(Tok::RParen) {
                        let fname = self.ident()?;
                        self.expect(Tok::Colon)?;
                        let fty = self.ty()?;
                        fields.push((fname, fty));
                        if !self.eat(Tok::Comma) {
                            self.expect(Tok::RParen)?;
                            break;
                        }
                    }
                }
                variants.push(Variant {
                    name: vname,
                    fields,
                });
                self.eat(Tok::Comma);
            }
            return Ok(Decl::Enum(EnumDecl { name, variants }));
        }
        if self.eat(Tok::Fn) || self.eat(Tok::Private) {
            let private = self.toks[self.pos - 1] == Tok::Private;
            if private {
                self.expect(Tok::Fn)?;
            }
            return self.fn_decl(private).map(Decl::Fn);
        }
        Err(format!("期望声明，实际 {:?}", self.peek()))
    }

    fn fn_decl(&mut self, private: bool) -> Result<FnDecl, String> {
        let name = self.ident()?;
        self.expect(Tok::LParen)?;
        let mut params = Vec::new();
        while !self.eat(Tok::RParen) {
            let pname = self.ident()?;
            self.expect(Tok::Colon)?;
            let pty = self.ty()?;
            params.push((pname, pty));
            if !self.eat(Tok::Comma) {
                self.expect(Tok::RParen)?;
                break;
            }
        }
        let mut ret = None;
        if self.eat(Tok::Arrow) {
            ret = Some(self.ty()?);
        }
        let mut effects = Vec::new();
        loop {
            effects.push(match self.peek() {
                Tok::Ident(s) if s == "io" => Effect::Io,
                Tok::Ident(s) if s == "async" => Effect::Async,
                Tok::Ident(s) if s == "random" => Effect::Random,
                Tok::Ident(s) if s == "time" => Effect::Time,
                _ => break,
            });
            self.bump();
        }
        let mut body_part = None;
        if self.eat(Tok::Spec) {
            body_part = Some(BodyPart::Spec(self.spec_block()?));
        } else if self.eat(Tok::Trusted) {
            self.expect(Tok::LParen)?;
            let reason = match self.bump() {
                Tok::Str(s) => s,
                _ => return Err("trusted 需要一个字符串原因".into()),
            };
            self.expect(Tok::RParen)?;
            body_part = Some(BodyPart::Trusted(reason));
        }
        let body = self.block()?;
        Ok(FnDecl {
            private,
            name,
            params,
            ret,
            effects,
            body_part,
            body,
        })
    }

    fn spec_block(&mut self) -> Result<SpecBlock, String> {
        self.expect(Tok::LBrace)?;
        let mut b = SpecBlock::default();
        while !self.eat(Tok::RBrace) {
            if self.eat(Tok::Pre) {
                b.pres.push(self.expr()?);
            } else if self.eat(Tok::Post) {
                b.posts.push(self.expr()?);
            } else if self.eat(Tok::Modifies) {
                loop {
                    b.modifies.push(self.ident()?);
                    if !self.eat(Tok::Comma) {
                        break;
                    }
                }
            } else if self.eat(Tok::Examples) {
                self.expect(Tok::LBrace)?;
                while !self.eat(Tok::RBrace) {
                    let lhs = self.add()?;
                    if self.eat(Tok::EqEq) {
                        let rhs = self.expr()?;
                        b.examples.push(Example::Eq(lhs, rhs));
                    } else if self.eat(Tok::Is) {
                        let pat = self.pattern()?;
                        b.examples.push(Example::Is(lhs, pat));
                    } else {
                        return Err("example 需要 == 或 is".into());
                    }
                }
            } else {
                return Err(format!("spec 内期望 pre/post/modifies/examples，实际 {:?}", self.peek()));
            }
        }
        Ok(b)
    }

    fn block(&mut self) -> Result<Block, String> {
        self.expect(Tok::LBrace)?;
        let mut b = Block::default();
        let mut tail = None;
        while !self.eat(Tok::RBrace) {
            if self.eat(Tok::Let) {
                let name = self.ident()?;
                let mut ty = None;
                if self.eat(Tok::Colon) {
                    ty = Some(self.ty()?);
                }
                self.expect(Tok::Assign)?;
                let value = self.expr()?;
                self.expect(Tok::Semi)?;
                b.stmts.push(Stmt::Let { name, ty, value });
            } else if self.eat(Tok::Return) {
                if self.eat(Tok::Semi) {
                    b.stmts.push(Stmt::Return(None));
                } else {
                    let e = self.expr()?;
                    self.expect(Tok::Semi)?;
                    b.stmts.push(Stmt::Return(Some(e)));
                }
            } else if *self.peek() == Tok::RBrace {
                break;
            } else {
                let e = self.expr()?;
                if self.eat(Tok::Semi) {
                    b.stmts.push(Stmt::Expr(e));
                } else {
                    tail = Some(e);
                    self.expect(Tok::RBrace)?;
                    break;
                }
            }
        }
        b.tail = tail;
        Ok(b)
    }

    fn ty(&mut self) -> Result<Type, String> {
        if *self.peek() == Tok::Record {
            self.bump();
            self.expect(Tok::LBrace)?;
            let mut fields = Vec::new();
            while !self.eat(Tok::RBrace) {
                let fname = self.ident()?;
                self.expect(Tok::Colon)?;
                let fty = self.ty()?;
                fields.push((fname, fty));
                self.eat(Tok::Comma);
            }
            return Ok(Type::Record(fields));
        }
        let base = match self.bump() {
            Tok::IntKw => Type::Int,
            Tok::FloatKw => Type::Float,
            Tok::BoolKw => Type::Bool,
            Tok::StrKw => Type::Str,
            Tok::Ident(s) => {
                let mut args = Vec::new();
                if self.eat(Tok::Lt) {
                    while !self.eat(Tok::Gt) {
                        args.push(self.ty()?);
                        if !self.eat(Tok::Comma) {
                            self.expect(Tok::Gt)?;
                            break;
                        }
                    }
                }
                Type::Named(s, args)
            }
            other => return Err(format!("期望类型，实际 {:?}", other)),
        };
        if self.eat(Tok::Question) {
            Ok(Type::Opt(Box::new(base)))
        } else {
            Ok(base)
        }
    }

    fn pattern(&mut self) -> Result<Pattern, String> {
        match self.bump() {
            Tok::Underscore => Ok(Pattern::Wildcard),
            Tok::Int(n) => Ok(Pattern::Lit(Box::new(Expr::Int(n)))),
            Tok::Str(s) => Ok(Pattern::Lit(Box::new(Expr::Str(s)))),
            Tok::True => Ok(Pattern::Lit(Box::new(Expr::Bool(true)))),
            Tok::False => Ok(Pattern::Lit(Box::new(Expr::Bool(false)))),
            Tok::Ident(s) => {
                if self.eat(Tok::LParen) {
                    let mut inner = Vec::new();
                    while !self.eat(Tok::RParen) {
                        inner.push(self.pattern()?);
                        if !self.eat(Tok::Comma) {
                            self.expect(Tok::RParen)?;
                            break;
                        }
                    }
                    Ok(Pattern::Ctor(s, inner))
                } else if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    Ok(Pattern::Ctor(s, Vec::new()))
                } else {
                    Ok(Pattern::Bind(s))
                }
            }
            other => Err(format!("期望模式，实际 {:?}", other)),
        }
    }

    fn expr(&mut self) -> Result<Expr, String> {
        if self.eat(Tok::If) {
            let old = self.no_record;
            self.no_record = true;
            let cond = self.expr()?;
            self.no_record = old;
            let then = self.block()?;
            self.expect(Tok::Else)?;
            let els = self.block()?;
            return Ok(Expr::If(Box::new(cond), Box::new(then), Box::new(els)));
        }
        if self.eat(Tok::Match) {
            let old = self.no_record;
            self.no_record = true;
            let scrut = self.expr()?;
            self.no_record = old;
            self.expect(Tok::LBrace)?;
            let mut arms = Vec::new();
            while !self.eat(Tok::RBrace) {
                let pat = self.pattern()?;
                self.expect(Tok::FatArrow)?;
                let e = self.expr()?;
                arms.push((pat, e));
                self.eat(Tok::Comma);
            }
            return Ok(Expr::Match(Box::new(scrut), arms));
        }
        self.or()
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut lhs = self.and()?;
        while self.eat(Tok::OrOr) {
            let rhs = self.and()?;
            lhs = Expr::Binary(BinOp::Or, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut lhs = self.is_test()?;
        while self.eat(Tok::AndAnd) {
            let rhs = self.is_test()?;
            lhs = Expr::Binary(BinOp::And, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
    fn is_test(&mut self) -> Result<Expr, String> {
        let mut lhs = self.cmp()?;
        if self.eat(Tok::Is) {
            let pat = self.pattern()?;
            lhs = Expr::Is(Box::new(lhs), pat);
        }
        Ok(lhs)
    }
    fn cmp(&mut self) -> Result<Expr, String> {
        let mut lhs = self.add()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => BinOp::Eq,
                Tok::Ne => BinOp::Ne,
                Tok::Lt => BinOp::Lt,
                Tok::Le => BinOp::Le,
                Tok::Gt => BinOp::Gt,
                Tok::Ge => BinOp::Ge,
                _ => break,
            };
            self.bump();
            let rhs = self.add()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
    fn add(&mut self) -> Result<Expr, String> {
        let mut lhs = self.mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let rhs = self.mul()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
    fn mul(&mut self) -> Result<Expr, String> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Percent => BinOp::Mod,
                _ => break,
            };
            self.bump();
            let rhs = self.unary()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat(Tok::Minus) {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        if self.eat(Tok::Bang) {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?)));
        }
        self.postfix()
    }
    fn postfix(&mut self) -> Result<Expr, String> {
        let mut e = self.primary()?;
        loop {
            if self.eat(Tok::LParen) {
                let mut args = Vec::new();
                while !self.eat(Tok::RParen) {
                    args.push(self.expr()?);
                    if !self.eat(Tok::Comma) {
                        self.expect(Tok::RParen)?;
                        break;
                    }
                }
                e = Expr::Call(Box::new(e), args);
            } else if self.eat(Tok::Dot) {
                let name = self.ident()?;
                e = Expr::Field(Box::new(e), name);
            } else if self.eat(Tok::LBracket) {
                let idx = self.expr()?;
                self.expect(Tok::RBracket)?;
                e = Expr::Index(Box::new(e), Box::new(idx));
            } else {
                break;
            }
        }
        Ok(e)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Tok::Int(n) => Ok(Expr::Int(n)),
            Tok::Float(f) => Ok(Expr::Float(f)),
            Tok::True => Ok(Expr::Bool(true)),
            Tok::False => Ok(Expr::Bool(false)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Todo => Ok(Expr::Todo),
            Tok::Old => Ok(Expr::Ident("old".into())),
            Tok::Events => Ok(Expr::Ident("events".into())),
            Tok::Ident(s) => {
                if !self.no_record && self.eat(Tok::LBrace) {
                    let mut fields = Vec::new();
                    while !self.eat(Tok::RBrace) {
                        let fname = self.ident()?;
                        self.expect(Tok::Colon)?;
                        let fexpr = self.expr()?;
                        fields.push((fname, fexpr));
                        if !self.eat(Tok::Comma) {
                            self.expect(Tok::RBrace)?;
                            break;
                        }
                    }
                    Ok(Expr::RecordLit(s, fields))
                } else {
                    Ok(Expr::Ident(s))
                }
            }
            Tok::LParen => {
                let e = self.expr()?;
                self.expect(Tok::RParen)?;
                Ok(e)
            }
            Tok::LBracket => {
                let mut items = Vec::new();
                while !self.eat(Tok::RBracket) {
                    items.push(self.expr()?);
                    if !self.eat(Tok::Comma) {
                        self.expect(Tok::RBracket)?;
                        break;
                    }
                }
                Ok(Expr::List(items))
            }
            other => Err(format!("期望表达式，实际 {:?}", other)),
        }
    }
}
