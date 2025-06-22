// src/ast.rs

// Espressioni
#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Literal(String), // Per '3b010', 'x', etc.
    BinaryOp(Box<Expr>, Op, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Op { Plus, Minus, BitAnd, BitOr }

// Dichiarazioni all'interno di un modulo
#[derive(Debug, Clone)]
pub enum Declaration {
    Port {
        direction: PortDirection,
        is_reg: bool,
        name: String,
        width: u32,
    },
    Combinatorial(Vec<Statement>),
    // Blocco `gen if` che contiene altre dichiarazioni/statement
    ConditionalBlock {
        condition: String, // Condizione di generazione (es. "INCLUDE_LOGIC_OPS")
        declarations: Vec<Declaration>,
    }
}

#[derive(Debug, Clone)]
pub enum PortDirection { Input, Output }

// Statement all'interno di un blocco (es. combinatorial)
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Module(String, Vec<Statement>),
    VarDecl(String),
    Assignment(String, String), // Per ora, RHS è solo un identificatore
}

// La radice del nostro AST
#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<Declaration>,
}

