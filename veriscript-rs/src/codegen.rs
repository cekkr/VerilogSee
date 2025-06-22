// src/codegen.rs

use crate::ast::Statement;
use std::collections::HashMap;
use std::fmt::Write;

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn get_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
    
    // MODIFICATA la firma della funzione
    fn visit_statement(&mut self, statement: &Statement, scope: &mut HashMap<String, String>) {
        match statement {
            Statement::Module(name, statements) => {
                self.output.push_str(&format!("module {} {{\n", name));
                self.indent_level += 1;
                let mut inner_scope = HashMap::new(); // Crea un nuovo scope per il modulo
                for statement in statements {
                    // La chiamata ora è corretta
                    self.visit_statement(statement, &mut inner_scope);
                }
                self.indent_level -= 1;
                self.output.push_str("}\n");
            }
            Statement::VarDecl(name) => {
                // Esempio di utilizzo dello scope (può essere espanso)
                scope.insert(name.clone(), "wire".to_string());
                writeln!(self.output, "{}wire {};", self.get_indent(), name).unwrap();
            }
            Statement::Assignment(lhs, rhs) => {
                writeln!(
                    self.output,
                    "{}{} = {};",
                    self.get_indent(),
                    lhs,
                    rhs
                )
                .unwrap();
            }
        }
    }

    pub fn generate(&mut self, ast: &[Statement]) {
        // Crea lo scope globale e lo passa alla prima chiamata
        let mut global_scope = HashMap::new();
        for statement in ast {
            self.visit_statement(statement, &mut global_scope);
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }
}