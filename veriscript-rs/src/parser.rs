// src/parser.rs

use chumsky::prelude::*;
use crate::token::Token;
use crate::ast::*;

// Funzione principale che orchestra il parsing
pub fn parser() -> impl Parser<Token, Module, Error = Simple<Token>> {
    let ident = select! { Token::Identifier(s) => s }.labelled("identifier");

    // Parser per espressioni semplici (per ora non gestisce la precedenza)
    let expr = {
        let literal = select! {
            Token::Identifier(s) => Expr::Identifier(s),
            Token::BitVector(s) => Expr::Literal(s),
        };
        let op = select! {
            Token::Plus => Op::Plus, Token::Minus => Op::Minus,
            Token::BitAnd => Op::BitAnd, Token::BitOr => Op::BitOr,
        };
        literal.clone()
            .then(op.then(literal).repeated())
            .foldl(|lhs, (op, rhs)| Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs)))
    };

    // Parser per gli statement
    let statement = recursive(|stmt| {
        let assignment = ident
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(target, expr)| Statement::Assignment { target, expr });

        let case_stmt = just(Token::Case)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Colon))
            .then(stmt.clone())
            .map(|(case_expr, stmt)| (case_expr, Box::new(stmt)));
            
        let default_stmt = just(Token::Default)
            .ignore_then(just(Token::Colon))
            .ignore_then(stmt.clone())
            .map(Box::new);

        let switch = just(Token::Switch)
            .ignore_then(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
            .then(
                case_stmt.repeated()
                .then(default_stmt.or_not())
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
            )
            .map(|(switch_expr, (cases, default))| Statement::Switch { switch_expr, cases, default });
        
        assignment.or(switch)
    });

    // Parser per le dichiarazioni dentro un modulo
    let declaration = recursive(|decl| {
        let port_decl = just(Token::Port)
            .ignore_then(select! { Token::Input => PortDirection::Input, Token::Output => PortDirection::Output })
            .then(just(Token::Reg).or_not())
            .then(ident)
            .then(just(Token::LBracket).ignore_then(select!{Token::Integer(i) => i as u32}).ignore_then(just(Token::RBracket)))
            .then_ignore(just(Token::Semicolon))
            .map(|(((dir, is_reg), name), width)| Declaration::Port {
                direction: dir, is_reg: is_reg.is_some(), name, width
            });

        let combinatorial_block = just(Token::Combinatorial)
            .ignore_then(statement.repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(Declaration::Combinatorial);

        // Parsing di 'gen if'
        let gen_if_block = just(Token::Gen).ignore_then(just(Token::If))
            .ignore_then(ident.delimited_by(just(Token::LParen), just(Token::RParen)))
            .then(
                // Un `gen if` può contenere solo `case` statement per questo prototipo
                 just(Token::Case).ignore_then(expr.clone()).then_ignore(just(Token::Colon))
                .then(statement.clone())
                .map(|(case_expr, stmt)| Declaration::Combinatorial(vec![Statement::Switch {
                    switch_expr: Expr::Identifier("".to_string()), // Placeholder
                    cases: vec![(case_expr, Box::new(stmt))],
                    default: None,
                }]))
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
            )
            .map(|(condition, declarations)| Declaration::ConditionalBlock { condition, declarations });
        
        port_decl.or(combinatorial_block).or(gen_if_block)
    });
    
    // Parser del modulo completo
    just(Token::Module)
        .ignore_then(ident)
        .then(declaration.repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map(|(name, declarations)| Module { name, declarations })
        .then_ignore(end())
}