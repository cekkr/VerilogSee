use chumsky::prelude::*;
use std::env;
use std::error::Error;
use std::fs;

mod ast;
mod codegen;
mod parser;
mod token;

use crate::parser::module_parser;
use crate::token::{lexer, SimpleSpan};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("Usage: veridec <path>");
    let src = fs::read_to_string(&path)?;

    // 1. Esegui il Lexer
    let (tokens, lex_errs) = lexer().parse_recovery(&src);

    // 2. Esegui il Parser
    let (ast, parse_errs) = if let Some(tokens) = tokens {
        // ---- LA MODIFICA FONDAMENTALE È QUI ----
        // Creiamo uno stream che Chumsky capisce, a partire dal vettore di token.
        // Lo span finale (eoi) aiuta a segnalare errori di fine file inaspettata.
        let eoi = SimpleSpan::new(src.len(), src.len());
        let stream = chumsky::Stream::from_iter(tokens.into_iter()).spanned(eoi);
        
        module_parser().parse_recovery(stream)
    } else {
        // Se il lexing fallisce completamente, non c'è nulla da parsare
        (None, Vec::new())
    };

    // 3. Gestione e stampa degli errori
    // Combina gli errori del lexer e del parser
    let all_errors = lex_errs.into_iter().chain(parse_errs.into_iter());
    let mut error_found = false;
    for err in all_errors {
        error_found = true;
        // Qui si potrebbe usare una libreria come 'ariadne' per una stampa più bella
        println!("{:?}", err);
    }

    if error_found {
        return Err("Compilazione fallita.".into());
    }

    // 4. Successo
    if let Some(ast) = ast {
        println!("AST generato con successo:\n{:#?}", ast);
        // Qui puoi riattivare il CodeGenerator
    } else {
        println!("Nessun AST generato (il file sorgente potrebbe essere vuoto o contenere solo errori).");
    }

    Ok(())
}