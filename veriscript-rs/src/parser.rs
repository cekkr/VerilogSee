use chumsky::prelude::*;
use chumsky::recursive::recursive;

use crate::ast::*;
use crate::token::{SimpleSpan, VToken};

// Alias di tipo per rendere le firme più pulite e leggibili
type Span = SimpleSpan;
type Spanned<T> = (T, Span);
type TokenStream<'a> = &'a [Spanned<VToken>];
type ParserError<'a> = extra::Err<Rich<'a, VToken, Span>>;

// Funzione helper per estrarre un identificatore come stringa
fn ident<'a>() -> impl Parser<'a, TokenStream<'a>, String, ParserError<'a>> {
    select! { VToken::Ident(s) => s }.labelled("identifier")
}

// La firma della funzione ora è corretta e chiara.
// Il parser principale che definisce un modulo Verilog.
pub fn module_parser<'a>() -> impl Parser<'a, TokenStream<'a>, Module, ParserError<'a>> {
    // Un parser per dichiarazioni, definito ricorsivamente
    let declaration = recursive(|_decl| {
        // Esempio: una dichiarazione di porta (input/output)
        let port_decl = choice((just(VToken::Input), just(VToken::Output)))
            .then(just(VToken::Reg).or_not()) // opzionale [reg]
            .then(ident())
            .then_ignore(just(VToken::Semicolon))
            .map(|((dir, reg), name)| {
                let direction = if dir == VToken::Input { PortDirection::Input } else { PortDirection::Output };
                Declaration::Port(Port {
                    direction,
                    is_reg: reg.is_some(),
                    name,
                })
            });

        // Qui potresti aggiungere altri tipi di dichiarazioni con .or()
        // es: .or(assignment).or(wire_decl)
        port_decl
    });

    // Parser per un intero modulo
    just(VToken::Module)
        .ignore_then(ident())
        .then(declaration.repeated().collect::<Vec<_>>())
        .then_ignore(just(VToken::EndModule))
        .map(|(name, body)| Module { name, body })
        .then_ignore(end())
}