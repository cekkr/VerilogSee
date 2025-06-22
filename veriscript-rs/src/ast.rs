// src/ast.rs

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub body: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Port(Port),
    // Aggiungi qui altre dichiarazioni: Wire, Reg, Assign, etc.
}

#[derive(Debug, Clone)]
pub struct Port {
    pub direction: PortDirection,
    pub is_reg: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    Inout, // Aggiunto per completezza
}