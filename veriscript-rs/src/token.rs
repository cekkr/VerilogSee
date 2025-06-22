use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VToken {
    Module, EndModule, Port, Input, Output, Reg, Wire, Assign, Always,
    If, Else, Begin, End, Genvar, Generate, EndGenerate, For,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Semicolon, Comma, At, Pound,
    Eq, Neq, Lt, Gt, Lte, Gte, AssignEq,
    Ident(String),
    Number(String),
}

pub type SimpleSpan = chumsky::span::SimpleSpan<usize>;

pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<(VToken, SimpleSpan)>, extra::Err<Rich<'a, char>>> {
    let ident = text::ident().map(|s: &str| match s {
        "module" => VToken::Module, "endmodule" => VToken::EndModule, "port" => VToken::Port,
        "input" => VToken::Input, "output" => VToken::Output, "reg" => VToken::Reg,
        "wire" => VToken::Wire, "assign" => VToken::Assign, "always" => VToken::Always,
        "if" => VToken::If, "else" => VToken::Else, "begin" => VToken::Begin, "end" => VToken::End,
        "genvar" => VToken::Genvar, "generate" => VToken::Generate, "endgenerate" => VToken::EndGenerate,
        "for" => VToken::For,
        _ => VToken::Ident(s.to_string()),
    });

    let number = text::int(10).map(|s: &str| VToken::Number(s.to_string()));

    let punc = choice((
        just("==").to(VToken::Eq), just("!=").to(VToken::Neq), just("<=").to(VToken::Lte),
        just(">=").to(VToken::Gte), just("=").to(VToken::AssignEq), just("(").to(VToken::LParen),
        just(")").to(VToken::RParen), just("{").to(VToken::LBrace), just("}").to(VToken::RBrace),
        just("[").to(VToken::LBracket), just("]").to(VToken::RBracket), just(";").to(VToken::Semicolon),
        just(",").to(VToken::Comma), just("@").to(VToken::At), just("#").to(VToken::Pound),
        just("<").to(VToken::Lt), just(">").to(VToken::Gt),
    ));

    let token = punc.or(ident).or(number);

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect()
}