use chumsky::prelude::*;

// Aggiungo Eq e Hash, possono essere utili in futuro
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
    // Parser per identificatori e parole chiave
    let ident = text::ident().map(|s: String| match s.as_str() {
        "module" => VToken::Module,
        "endmodule" => VToken::EndModule,
        "port" => VToken::Port,
        "input" => VToken::Input,
        "output" => VToken::Output,
        "reg" => VToken::Reg,
        "wire" => VToken::Wire,
        "assign" => VToken::Assign,
        "always" => VToken::Always,
        "if" => VToken::If,
        "else" => VToken::Else,
        "begin" => VToken::Begin,
        "end" => VToken::End,
        "genvar" => VToken::Genvar,
        "generate" => VToken::Generate,
        "endgenerate" => VToken::EndGenerate,
        "for" => VToken::For,
        _ => VToken::Ident(s),
    });

    // Parser per numeri
    let number = text::int(10).map(VToken::Number);

    // Parser per operatori e punteggiatura
    let punc = choice((
        just("==").to(VToken::Eq),
        just("!=").to(VToken::Neq),
        just("<=").to(VToken::Lte),
        just(">=").to(VToken::Gte),
        just("=").to(VToken::AssignEq),
        just("(").to(VToken::LParen),
        just(")").to(VToken::RParen),
        just("{").to(VToken::LBrace),
        just("}").to(VToken::RBrace),
        just("[").to(VToken::LBracket),
        just("]").to(VToken::RBracket),
        just(";").to(VToken::Semicolon),
        just(",").to(VToken::Comma),
        just("@").to(VToken::At),
        just("#").to(VToken::Pound),
        just("<").to(VToken::Lt),
        just(">").to(VToken::Gt),
    ));

    // Un singolo token è una delle tre categorie precedenti
    let token = punc.or(ident).or(number);

    // Il lexer completo mappa ogni token al suo span, ignora gli spazi
    // e raccoglie tutto in un vettore.
    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .repeated()
        .collect()
}