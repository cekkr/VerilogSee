// src/token.rs
use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*(?:[^*]|\*[^/])*\*?/")]
pub enum Token {
    #[token("module")] Module,
    #[token("port")] Port,
    #[token("input")] Input,
    #[token("output")] Output,
    #[token("wire")] Wire,
    #[token("reg")] Reg,
    #[token("combinatorial")] Combinatorial,
    #[token("sequential")] Sequential,
    #[token("switch")] Switch,
    #[token("case")] Case,
    #[token("default")] Default,
    #[token("gen")] Gen,
    #[token("if")] If,
    #[token("import")] Import,
    #[token("param")] Param,
    #[token("=")] Assign,
    #[token("<-")] NonBlockingAssign,
    #[token("==")] Eq,
    #[token("+")] Plus, #[token("-")] Minus,
    #[token("&")] BitAnd, #[token("|")] BitOr,
    #[token("^")] BitXor, #[token("<<")] ShiftLeft,
    #[token("{")] LBrace, #[token("}")] RBrace,
    #[token("[")] LBracket, #[token("]")] RBracket,
    #[token("(")] LParen, #[token(")")] RParen,
    #[token(":")] Colon, #[token(";")] Semicolon,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex("'[0-9]*[bBhdD][a-fA-F0-9_xXzZ]+", |lex| lex.slice().to_string())]
    BitVector(String),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    StringLiteral(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}