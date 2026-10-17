#[derive(Debug, Clone)]
pub struct Module {
    pub name: Option<String>,
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Type(TypeDecl),
    Enum(EnumDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub private: bool,
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Option<Type>,
    pub effects: Vec<Effect>,
    pub body_part: Option<BodyPart>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Io,
    Async,
    Random,
    Time,
}

#[derive(Debug, Clone)]
pub enum BodyPart {
    Spec(SpecBlock),
    Trusted(String),
}

#[derive(Debug, Clone, Default)]
pub struct SpecBlock {
    pub pres: Vec<Expr>,
    pub posts: Vec<Expr>,
    pub modifies: Vec<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone)]
pub enum Example {
    Eq(Expr, Expr),
    Is(Expr, Pattern),
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Record(Vec<(String, Type)>),
    Named(String, Vec<Type>),
    Opt(Box<Type>),
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        value: Expr,
    },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(String),
    Call(Box<Expr>, Vec<Expr>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    RecordLit(String, Vec<(String, Expr)>),
    List(Vec<Expr>),
    If(Box<Expr>, Box<Block>, Box<Block>),
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
    Is(Box<Expr>, Pattern),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Todo,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Lit(Box<Expr>),
    Bind(String),
    Ctor(String, Vec<Pattern>),
}
