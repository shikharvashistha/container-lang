#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Service, Image, Replicas, Ports, Env, Volumes,
    LBrace, RBrace, Colon, Comma, Eq,
    Ident(String), Number(u32), StringLit(String),
    Eof,
}

pub struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self { chars: src.chars().collect(), i: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> { self.chars.get(self.i).copied() }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += 1;
        if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        Some(c)
    }

    fn skip_ws_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) { self.bump(); }
            if self.peek() == Some('#') {
                while let Some(c) = self.bump() { if c == '\n' { break; } }
                continue;
            }
            break;
        }
    }

    fn lex_ident_or_kw(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' { s.push(c); self.bump(); } else { break; }
        }
        match s.as_str() {
            "service" => Token::Service,
            "image" => Token::Image,
            "replicas" => Token::Replicas,
            "ports" => Token::Ports,
            "env" => Token::Env,
            "volumes" => Token::Volumes,
            _ => Token::Ident(s),
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { s.push(c); self.bump(); } else { break; }
        }
        let n: u32 = s.parse().unwrap();
        Token::Number(n)
    }

    fn lex_string(&mut self) -> Result<Token, String> {
        // opening quote already consumed
        let mut s = String::new();
        loop {
            match self.bump() {
                Some('"') => break,
                Some('\\') => {
                    match self.bump() {
                        Some('"') => s.push('"'),
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some(other) => { s.push('\\'); s.push(other); }
                        None => return Err("unterminated escape".into()),
                    }
                }
                Some(c) => s.push(c),
                None => return Err("unterminated string".into()),
            }
        }
        Ok(Token::StringLit(s))
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_ws_comments();
        let c = match self.peek() { Some(c) => c, None => return Ok(Token::Eof) };
        Ok(match c {
            '{' => { self.bump(); Token::LBrace }
            '}' => { self.bump(); Token::RBrace }
            ':' => { self.bump(); Token::Colon }
            ',' => { self.bump(); Token::Comma }
            '=' => { self.bump(); Token::Eq }
            '"' => { self.bump(); self.lex_string()? }
            c if c.is_ascii_digit() => self.lex_number(),
            c if c.is_alphabetic() || c == '_' => self.lex_ident_or_kw(),
            _ => return Err(format!("unexpected character '{}' at {}:{}", c, self.line, self.col)),
        })
    }
}
