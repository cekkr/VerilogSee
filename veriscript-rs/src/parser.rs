use chumsky::prelude::*;
use chumsky::recursive::recursive;

use crate::ast::*;
use crate::token::{SimpleSpan, VToken};

type TokenStream<'a> = &'a [(VToken, SimpleSpan)];
type ParserError<'a> = extra::Err<Rich<'a, (VToken, SimpleSpan)>>;

// Parser ausiliario per un token specifico.
fn just<'a>(token: VToken) -> impl Parser<'a, TokenStream<'a>, (VToken, SimpleSpan), ParserError<'a>> + Clone {
    any().filter(move |(t, _)| *t == token)
}

// Parser ausiliario per un identificatore.
fn ident<'a>() -> impl Parser<'a, TokenStream<'a>, String, ParserError<'a>> + Clone {
    any().try_map(|(token, span)| match token {
        VToken::Ident(s) => Ok(s),
        _ => Err(Rich::custom(span, "Expected identifier")),
    })
}

// Parser ausiliario per una direzione di porta.
fn port_direction<'a>() -> impl Parser<'a, TokenStream<'a>, PortDirection, ParserError<'a>> + Clone {
    any().try_map(|(token, span)| match token {
        VToken::Input => Ok(PortDirection::Input),
        VToken::Output => Ok(PortDirection::Output),
        _ => Err(Rich::custom(span, "Expected 'input' or 'output'")),
    })
}

pub fn module_parser<'a>() -> impl Parser<'a, TokenStream<'a>, Module, ParserError<'a>> {
    let declaration = recursive(|_declaration| {
        port_direction()
            .then(just(VToken::Reg).or_not())
            .then(ident())
            .then_ignore(just(VToken::Semicolon))
            .map(|((direction, is_reg), name)| {
                Declaration::Port(Port {
                    direction,
                    is_reg: is_reg.is_some(),
                    name,
                })
            })
    });

    just(VToken::Module)
        .ignore_then(ident())
        .then(
            declaration
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(VToken::LParen), just(VToken::RParen)),
        )
        .then_ignore(just(VToken::Semicolon))
        // Ho rimosso la strategia di recovery errata, come suggerito dal compilatore
        .then_ignore(end())
        .map(|(name, body)| Module { name, body })
}

