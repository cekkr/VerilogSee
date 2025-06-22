use chumsky::prelude::*;
use std::env;
use std::error::Error;
use std::fs;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

mod ast;
mod parser;
mod token;

use crate::parser::module_parser;
use crate::token::VToken;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("Usage: veridec <path>");
    let src = fs::read_to_string(&path)?;

    // --- 1. LEXER ---
    let (tokens, lex_errs) = token::lexer().parse(src.as_str()).into_output_errors();

    // --- 2. PARSER ---
    if let Some(tokens) = tokens {
        let (ast, parse_errs) = module_parser().parse(&tokens).into_output_errors();

        // --- 3. GESTIONE ERRORI CON ARIADNE ---
        lex_errs.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, &path, e.span().start)
                .with_message("Lexer Error") // Messaggio generico
                .with_label(
                    Label::new((&path, e.span().into_range()))
                        .with_message(format!("Reason: {:?}", e.reason())) // Formattazione Debug
                        .with_color(Color::Red),
                )
                .finish()
                .print((&path, Source::from(&src)))
                .unwrap();
        });

        parse_errs.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, &path, e.span().start)
                .with_message("Parser Error") // Messaggio generico
                .with_label(
                    Label::new((&path, e.span().into_range()))
                        .with_message(format!("Reason: {:?}", e.reason())) // Formattazione Debug
                        .with_color(Color::Red),
                )
                .finish()
                .print((&path, Source::from(&src)))
                .unwrap();
        });

        // --- 4. SUCCESSO ---
        if let Some(ast) = ast {
            println!("AST generato con successo:\n{:#?}", ast);
        }
    }

    Ok(())
}