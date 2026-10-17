#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),

    Module,
    Type,
    Record,
    Enum,
    Fn,
    Spec,
    Pre,
    Post,
    Examples,
    If,
    Else,
    Match,
    Is,
    Old,
    Modifies,
    Events,
    Let,
    Return,
    Todo,
    True,
    False,
    Private,
    Trusted,

    IntKw,
    FloatKw,
    BoolKw,
    StrKw,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Arrow,
    FatArrow,
    Question,
    Underscore,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    Assign,
    Semi,
    Eof,
}

pub fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let cs: Vec<char> = src.chars().collect();
    let mut i = 0;

    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < cs.len() && cs[i + 1] == '/' {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < cs.len() && cs[i + 1] == '*' {
            i += 2;
            while i + 1 < cs.len() && !(cs[i] == '*' && cs[i + 1] == '/') {
                i += 1;
            }
            if i + 1 >= cs.len() {
                return Err("未闭合的块注释".into());
            }
            i += 2;
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_') {
                i += 1;
            }
            let word: String = cs[start..i].iter().collect();
            out.push(keyword_or_ident(&word));
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            let mut is_float = false;
            if i < cs.len() && cs[i] == '.' {
                is_float = true;
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let text: String = cs[start..i].iter().collect();
            if is_float {
                out.push(Tok::Float(text.parse().map_err(|_| "非法浮点数".to_string())?));
            } else {
                out.push(Tok::Int(text.parse().map_err(|_| "非法整数".to_string())?));
            }
            continue;
        }

        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < cs.len() && cs[i] != '"' {
                if cs[i] == '\\' {
                    i += 1;
                    if i >= cs.len() {
                        return Err("未闭合的字符串".into());
                    }
                    match cs[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        other => return Err(format!("非法转义: \\{}", other)),
                    }
                } else {
                    s.push(cs[i]);
                }
                i += 1;
            }
            if i >= cs.len() {
                return Err("未闭合的字符串".into());
            }
            i += 1;
            out.push(Tok::Str(s));
            continue;
        }

        let two = if i + 1 < cs.len() {
            Some((cs[i], cs[i + 1]))
        } else {
            None
        };
        let t = match two {
            Some(('-', '>')) => {
                i += 1;
                Tok::Arrow
            }
            Some(('=', '>')) => {
                i += 1;
                Tok::FatArrow
            }
            Some(('=', '=')) => {
                i += 1;
                Tok::EqEq
            }
            Some(('!', '=')) => {
                i += 1;
                Tok::Ne
            }
            Some(('<', '=')) => {
                i += 1;
                Tok::Le
            }
            Some(('>', '=')) => {
                i += 1;
                Tok::Ge
            }
            Some(('&', '&')) => {
                i += 1;
                Tok::AndAnd
            }
            Some(('|', '|')) => {
                i += 1;
                Tok::OrOr
            }
            _ => match c {
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '{' => Tok::LBrace,
                '}' => Tok::RBrace,
                '[' => Tok::LBracket,
                ']' => Tok::RBracket,
                '<' => Tok::Lt,
                '>' => Tok::Gt,
                ',' => Tok::Comma,
                '.' => Tok::Dot,
                ':' => Tok::Colon,
                '?' => Tok::Question,
                '_' => Tok::Underscore,
                '+' => Tok::Plus,
                '-' => Tok::Minus,
                '*' => Tok::Star,
                '/' => Tok::Slash,
                '%' => Tok::Percent,
                '=' => Tok::Assign,
                ';' => Tok::Semi,
                '!' => Tok::Bang,
                other => return Err(format!("非法字符: {}", other)),
            },
        };
        out.push(t);
        i += 1;
    }
    out.push(Tok::Eof);
    Ok(out)
}

fn keyword_or_ident(word: &str) -> Tok {
    match word {
        "module" => Tok::Module,
        "type" => Tok::Type,
        "record" => Tok::Record,
        "enum" => Tok::Enum,
        "fn" => Tok::Fn,
        "spec" => Tok::Spec,
        "pre" => Tok::Pre,
        "post" => Tok::Post,
        "examples" => Tok::Examples,
        "if" => Tok::If,
        "else" => Tok::Else,
        "match" => Tok::Match,
        "is" => Tok::Is,
        "old" => Tok::Old,
        "modifies" => Tok::Modifies,
        "events" => Tok::Events,
        "let" => Tok::Let,
        "return" => Tok::Return,
        "todo" => Tok::Todo,
        "true" => Tok::True,
        "false" => Tok::False,
        "private" => Tok::Private,
        "trusted" => Tok::Trusted,
        "Int" => Tok::IntKw,
        "Float" => Tok::FloatKw,
        "Bool" => Tok::BoolKw,
        "String" => Tok::StrKw,
        _ => Tok::Ident(word.to_string()),
    }
}
