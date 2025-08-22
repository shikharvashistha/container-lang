use crate::ir::{Program, Service};
use crate::lexer::Token;
use crate::lexer::Lexer;

pub struct Parser {
    lex: Lexer,
    cur: Token,
}

impl Parser {
    pub fn new(mut lex: Lexer) -> Result<Self, String> {
        let cur = lex.next_token()?;
        Ok(Self { lex, cur })
    }

    fn bump(&mut self) -> Result<(), String> {
        self.cur = self.lex.next_token()?;
        Ok(())
    }

    fn expect(&mut self, t: &Token) -> Result<(), String> {
        if &self.cur == t { self.bump() } else { Err(format!("expected {:?}, found {:?}", t, self.cur)) }
    }

    fn take_ident(&mut self) -> Result<String, String> {
        if let Token::Ident(s) = &self.cur { let out = s.clone(); self.bump()?; Ok(out) }
        else { Err(format!("expected identifier, found {:?}", self.cur)) }
    }

    fn take_number(&mut self) -> Result<u32, String> {
        if let Token::Number(n) = self.cur { self.bump()?; Ok(n) }
        else { Err(format!("expected number, found {:?}", self.cur)) }
    }

    fn take_string_or_ident(&mut self) -> Result<String, String> {
        match &self.cur {
            Token::StringLit(s) => { let out = s.clone(); self.bump()?; Ok(out) }
            Token::Ident(s) => { let out = s.clone(); self.bump()?; Ok(out) }
            _ => Err(format!("expected string or ident, found {:?}", self.cur)),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut services = Vec::new();
        loop {
            match self.cur {
                Token::Service => services.push(self.parse_service()?),
                Token::Eof => break,
                _ => return Err(format!("expected 'service' or EOF, found {:?}", self.cur)),
            }
        }
        Ok(Program { services })
    }

    fn parse_service(&mut self) -> Result<Service, String> {
        self.expect(&Token::Service)?;
        let name = self.take_ident()?;
        self.expect(&Token::LBrace)?;
        let mut svc = Service { name, replicas: 1, ..Default::default() };
        while self.cur != Token::RBrace {
            match self.cur {
                Token::Image => {
                    self.bump()?;
                    let img = self.take_string_or_ident()?;
                    svc.image = Some(img);
                }
                Token::Replicas => {
                    self.bump()?;
                    let n = self.take_number()?;
                    svc.replicas = n;
                }
                Token::Ports => {
                    self.bump()?;
                    loop {
                        let h = self.take_number()? as u16;
                        self.expect(&Token::Colon)?;
                        let c = self.take_number()? as u16;
                        svc.ports.push((h, c));
                        if self.cur == Token::Comma { self.bump()?; continue; } else { break; }
                    }
                }
                Token::Env => {
                    self.bump()?;
                    loop {
                        let key = self.take_ident()?;
                        self.expect(&Token::Eq)?;
                        let val = self.take_string_or_ident()?;
                        svc.env.push((key, val));
                        if self.cur == Token::Comma { self.bump()?; continue; } else { break; }
                    }
                }
                Token::Volumes => {
                    self.bump()?;
                    loop {
                        let path = match &self.cur {
                            Token::StringLit(s) => { let out = s.clone(); self.bump()?; out }
                            _ => return Err(format!("volume mapping must be a quoted string, found {:?}", self.cur)),
                        };
                        svc.volumes.push(path);
                        if self.cur == Token::Comma { self.bump()?; continue; } else { break; }
                    }
                }
                _ => return Err(format!("unexpected token in service block: {:?}", self.cur)),
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(svc)
    }
}
