mod lexer;
mod parser;
mod ir;
mod exec;

use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use exec::{execute, ExecOptions};

fn usage_and_exit() -> ! {
    eprintln!("usage: contc [--plan] <input.container>");
    process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() { usage_and_exit() }

    let mut plan_only = false;
    if args[0] == "--plan" {
        plan_only = true;
        args.remove(0);
    }
    if args.len() != 1 { usage_and_exit() }
    let path = &args[0];

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("error: cannot read {}: {e}", path); process::exit(1); }
    };

    // Frontend: lex + parse
    let lex = Lexer::new(&src);
    let mut parser = match Parser::new(lex) {
        Ok(p) => p,
        Err(e) => { eprintln!("lexer error: {e}"); process::exit(1); }
    };
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => { eprintln!("parse error: {e}"); process::exit(1); }
    };

    // Semantic checks
    if let Err(e) = program.validate() {
        eprintln!("semantic error: {e}");
        process::exit(1);
    }

    // Backend: execute
    if let Err(e) = execute(&program, ExecOptions { plan_only }) {
        eprintln!("execution error: {e}");
        process::exit(1);
    }
}
