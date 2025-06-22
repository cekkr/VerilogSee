use chumsky::prelude::*;
use std::env;
use std::error::Error;
use std::fs;
// Ariadne è ottimo per stampare errori leggibili
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

mod ast;
mod codegen;
mod parser;
mod token;

use crate::parser::module_parser;
use crate::token::{lexer, SimpleSpan, VToken};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("Usage: veridec <path>");
    let src = fs::read_to_string(&path)?;

    // --- 1. LEXER ---
    let (tokens, lex_errs) = lexer().parse_recovery(&src);

    // --- 2. PARSER ---
    let (ast, parse_errs) = if let Some(tokens) = tokens {
        // ---- LA CORREZIONE FONDAMENTALE È QUI ----
        // Trasformiamo il `Vec<(VToken, SimpleSpan)>` in uno `Stream`
        // che Chumsky può usare. Questo risolve TUTTI gli errori.
        let stream = chumsky::Stream::from_iter(tokens.into_iter())
            .spanned(SimpleSpan::new(src.len(), src.len()));
        
        module_parser().parse_recovery(stream)
    } else {
        (None, Vec::new())
    };

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

    Ok(())
}