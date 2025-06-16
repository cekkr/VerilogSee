// src/main.rs

mod token;
mod ast;
mod parser;
mod codegen;

use logos::Logos;
use chumsky::Parser;
use std::fs;

fn main() {
    println!("--- Veride Compiler (veridec) v0.1 ---");

    // Leggi il codice sorgente da un file
    // Crea un file `source.vd` nella root del progetto col codice di esempio.
    let source_code = fs::read_to_string("source.vd")
        .expect("Impossibile leggere il file source.vd");

    // --- Fase 1: Lexing ---
    let lexer = token::Token::lexer(&source_code);
    let tokens: Vec<_> = lexer.filter_map(|try_token| try_token.ok()).collect();
    
    // --- Fase 2: Parsing ---
    let ast_result = parser::parser().parse(tokens);

    match ast_result {
        Ok(ast) => {
            println!("Parsing completato con successo. AST generato.");
            
            // --- Fase 3: Code Generation ---
            println!("Inizio generazione codice Verilog...");
            let mut generator = codegen::CodeGenerator::new();
            let verilog_code = generator.generate(&ast);
            
            println!("\n--- OUTPUT VERILOG ---\n");
            println!("{}", verilog_code);

            // Scrivi l'output su file
            fs::write("output.v", verilog_code).expect("Impossibile scrivere il file di output.");
            println!("\n--- Scritto su output.v ---");
        }
        Err(errors) => {
            eprintln!("\nErrore durante il parsing:");
            for error in errors {
                eprintln!("- {}", error);
            }
        }
    }
}