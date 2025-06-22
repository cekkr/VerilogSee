use chumsky::prelude::*;
use chumsky::recursive::recursive;
use chumsky::stream::Stream; // Import the Stream type

use crate::ast::*;
use crate::token::{SimpleSpan, VToken};

// --- Tipi per chiarezza ---
type Span = SimpleSpan;
type Spanned<T> = (T, Span);
// Define TokenStream as a slice of spanned tokens
type TokenStream<'a> = &'a [Spanned<VToken>];

// Usiamo un errore che riconosce i VToken, non più i caratteri
type ParserError<'a> = extra::Err<Rich<'a, VToken>>;

fn ident<'a>() -> impl Parser<'a, TokenStream<'a>, String, ParserError<'a>> {
    select! { VToken::Ident(s) => s }.labelled("identifier")
}

pub fn module_parser<'a>() -> impl Parser<'a, TokenStream<'a>, Module, ParserError<'a>> {
    let declaration = recursive(|_decl| {
        let port_decl = select! { VToken::Input => PortDirection::Input, VToken::Output => PortDirection::Output }
            .then(select!{ VToken::Reg }.or_not())
            .then(ident())
            .then_ignore(select!{ VToken::Semicolon })
            .map(|((direction, is_reg), name)| {
                Declaration::Port(Port {
                    direction,
                    is_reg: is_reg.is_some(),
                    name,
                })
            });

        port_decl // Qui aggiungerai .or(altra_dichiarazione)
    });

    select!{VToken::Module}
        .ignore_then(ident())
        .then(declaration.repeated().collect::<Vec<_>>().delimited_by(select!{VToken::LParen}, select!{VToken::RParen}))
        .then_ignore(select!{VToken::Semicolon})
        .then_ignore(end()) // Assicurati che non ci sia altro dopo
        .map(|(name, body)| Module { name, body })
}