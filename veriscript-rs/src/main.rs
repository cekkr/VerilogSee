use chumsky::prelude::*;
use std::env;
use std::error::Error;
use std::fs;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

mod ast;
mod parser;
mod token;

use crate::parser::module_parser;
use crate::token::{lexer, SimpleSpan, VToken};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("Usage: veridec <path>");
    let src = fs::read_to_string(&path)?;

    // --- 1. LEXER ---
    let (tokens, lex_errs) = lexer().parse_recovery(src.as_str());

    // --- 2. PARSER ---
    if let Some(tokens) = &tokens {
        let len = src.chars().count();
        // Create a stream from the tokens that the parser can use.
        let stream = chumsky::stream::Stream::from_iter(tokens.clone().into_iter()).spanned(len..len + 1);
        let (ast, parse_errs) = module_parser().parse_recovery(stream);

        // --- 3. GESTIONE ERRORI CON ARIADNE ---
        // Stampa errori del lexer (basati su caratteri)
        for e in lex_errs {
            Report::build(ReportKind::Error, &path, e.span().start)
                .with_message(e.to_string())
                .with_label(Label::new((&path, e.span().into_range())).with_message(e.reason().to_string()).with_color(Color::Red))
                .finish()
                .print((&path, Source::from(&src)))?;
        }

        // Stampa errori del parser (basati su token)
        for e in parse_errs {
            Report::build(ReportKind::Error, &path, e.span().start)
                .with_message(e.to_string())
                .with_label(Label::new((&path, e.span().into_range())).with_message(e.reason().to_string()).with_color(Color::Red))
                .finish()
                .print((&path, Source::from(&src)))?;
        }

        // --- 4. SUCCESSO ---
        if let Some(ast) = ast {
            println!("AST generato con successo:\n{:#?}", ast);
        }
    }

    Ok(())
}