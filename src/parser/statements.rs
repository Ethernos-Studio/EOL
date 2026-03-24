//! 语句解析

use crate::ast::*;
use crate::error::cayResult;
use super::Parser;
use super::types::{parse_type, is_primitive_type_token};
use super::expressions::parse_expression;

/// 给语句添加标签
fn add_label_to_stmt(stmt: Stmt, label: String) -> Stmt {
    match stmt {
        Stmt::For(mut for_stmt) => {
            for_stmt.label = Some(label);
            Stmt::For(for_stmt)
        }
        Stmt::While(mut while_stmt) => {
            while_stmt.label = Some(label);
            Stmt::While(while_stmt)
        }
        Stmt::DoWhile(mut do_while_stmt) => {
            do_while_stmt.label = Some(label);
            Stmt::DoWhile(do_while_stmt)
        }
        _ => stmt, // 非循环语句不支持标签，保持原样
    }
}

/// 解析代码块
pub fn parse_block(parser: &mut Parser) -> cayResult<Block> {
    let loc = parser.current_loc();
    parser.consume(&crate::lexer::Token::LBrace, "Expected '{' to start block")?;
    
    let mut statements = Vec::new();
    while !parser.check(&crate::lexer::Token::RBrace) && !parser.is_at_end() {
        statements.push(parse_statement(parser)?);
    }
    
    parser.consume(&crate::lexer::Token::RBrace, "Expected '}' to end block")?;
    
    Ok(Block { statements, loc })
}

/// 解析语句
pub fn parse_statement(parser: &mut Parser) -> cayResult<Stmt> {
    // 检查是否是标签语句: label:
    if let crate::lexer::Token::Identifier(label_name) = parser.current_token().clone() {
        // 向前看检查是否是冒号
        let checkpoint = parser.pos;
        parser.advance(); // 跳过标识符
        
        if parser.check(&crate::lexer::Token::Colon) {
            parser.advance(); // 跳过冒号
            
            // 解析带标签的语句
            let stmt = parse_statement(parser)?;
            
            // 给语句添加标签
            return Ok(add_label_to_stmt(stmt, label_name));
        } else {
            // 不是标签，回退
            parser.pos = checkpoint;
        }
    }
    
    match parser.current_token() {
        crate::lexer::Token::LBrace => Ok(Stmt::Block(parse_block(parser)?)),
        crate::lexer::Token::If => parse_if_statement(parser),
        crate::lexer::Token::While => parse_while_statement(parser),
        crate::lexer::Token::For => parse_for_statement(parser),
        crate::lexer::Token::Do => parse_do_while_statement(parser),
        crate::lexer::Token::Switch => parse_switch_statement(parser),
        crate::lexer::Token::Scope => parse_scope_statement(parser),
        crate::lexer::Token::Return => parse_return_statement(parser),
        crate::lexer::Token::Break => {
            let _loc = parser.current_loc();
            parser.advance();
            
            // 检查是否有标签
            let label = if let crate::lexer::Token::Identifier(name) = parser.current_token().clone() {
                parser.advance();
                Some(name)
            } else {
                None
            };
            
            parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after break")?;
            Ok(Stmt::Break(label))
        }
        crate::lexer::Token::Continue => {
            let _loc = parser.current_loc();
            parser.advance();
            
            // 检查是否有标签
            let label = if let crate::lexer::Token::Identifier(name) = parser.current_token().clone() {
                parser.advance();
                Some(name)
            } else {
                None
            };
            
            parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after continue")?;
            Ok(Stmt::Continue(label))
        }
        _ => {
            // 检查是否是变量声明：支持任意类型标识（类名或原始类型），
            // 但要确保接下来的 token 是变量名（Identifier），以避免将函数调用等标识误判为类型。
            if parser.check(&crate::lexer::Token::Final) {
                return parse_var_decl(parser);
            }

            // 检查是否是 var/let/auto 后置类型声明
            if parser.check(&crate::lexer::Token::Var)
                || parser.check(&crate::lexer::Token::Let)
                || parser.check(&crate::lexer::Token::Auto)
            {
                return parse_var_decl(parser);
            }

            if super::types::is_type_token(parser) {
                // 尝试解析类型（不消耗最终位置）以判断是否紧跟变量名。
                let checkpoint = parser.pos;
                if super::types::parse_type(parser).is_ok() {
                    // 如果解析类型后当前token是标识符，则认为是变量声明
                    if let crate::lexer::Token::Identifier(_) = parser.current_token() {
                        parser.pos = checkpoint; // 回退到类型前位置
                        return parse_var_decl(parser);
                    }
                }
                // 回退到初始位置，继续解析为表达式语句
                parser.pos = checkpoint;
            }

            parse_expression_statement(parser)
        }
    }
}

/// 解析变量声明
/// 支持以下语法：
/// - 传统语法: int x = 10;
/// - final 修饰: final int x = 10;
/// - var 后置类型: var x: int = 10;
/// - let 后置类型: let x: int = 10;
/// - auto 类型推断: auto a = 42;
pub fn parse_var_decl(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();

    let is_final = parser.match_token(&crate::lexer::Token::Final);

    // 检查是否是 var/let/auto 语法
    let var_type = if parser.check(&crate::lexer::Token::Var)
        || parser.check(&crate::lexer::Token::Let)
        || parser.check(&crate::lexer::Token::Auto)
    {
        // var/let/auto 语法: var name: type = value; 或 auto name = value;
        parser.advance(); // 消费 var/let/auto

        // 解析变量名
        let name = parser.consume_identifier("Expected variable name after var/let/auto")?;

        // 检查是否有类型注解 (: type)
        let explicit_type = if parser.match_token(&crate::lexer::Token::Colon) {
            Some(parse_type(parser)?)
        } else {
            None
        };

        // 检查是否有初始化表达式
        let initializer = if parser.match_token(&crate::lexer::Token::Assign) {
            Some(parse_expression(parser)?)
        } else {
            None
        };

        parser.consume(
            &crate::lexer::Token::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        // 确定变量类型
        let var_type = match explicit_type {
            Some(t) => t,
            None => {
                // 如果没有显式类型，需要根据初始化表达式推断
                // 使用 Auto 类型作为占位符，由语义分析阶段推断
                crate::types::Type::Auto
            }
        };

        return Ok(Stmt::VarDecl(VarDecl {
            name,
            var_type,
            initializer,
            is_final,
            loc,
        }));
    } else {
        // 传统语法: type name = value;
        parse_type(parser)?
    };

    let name = parser.consume_identifier("Expected variable name")?;

    let initializer = if parser.match_token(&crate::lexer::Token::Assign) {
        Some(parse_expression(parser)?)
    } else {
        None
    };

    parser.consume(
        &crate::lexer::Token::Semicolon,
        "Expected ';' after variable declaration",
    )?;

    Ok(Stmt::VarDecl(VarDecl {
        name,
        var_type,
        initializer,
        is_final,
        loc,
    }))
}

/// 解析 if 语句
pub fn parse_if_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'if'
    
    parser.consume(&crate::lexer::Token::LParen, "Expected '(' after 'if'")?;
    let condition = parse_expression(parser)?;
    parser.consume(&crate::lexer::Token::RParen, "Expected ')' after if condition")?;
    
    let then_branch = Box::new(parse_statement(parser)?);
    let else_branch = if parser.match_token(&crate::lexer::Token::Else) {
        Some(Box::new(parse_statement(parser)?))
    } else {
        None
    };
    
    Ok(Stmt::If(IfStmt {
        condition,
        then_branch,
        else_branch,
        loc,
    }))
}

/// 解析 while 语句
pub fn parse_while_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'while'
    
    parser.consume(&crate::lexer::Token::LParen, "Expected '(' after 'while'")?;
    let condition = parse_expression(parser)?;
    parser.consume(&crate::lexer::Token::RParen, "Expected ')' after while condition")?;
    
    let body = Box::new(parse_statement(parser)?);
    
    Ok(Stmt::While(WhileStmt {
        condition,
        body,
        label: None,
        loc,
    }))
}

/// 解析 for 语句
pub fn parse_for_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'for'
    
    parser.consume(&crate::lexer::Token::LParen, "Expected '(' after 'for'")?;
    
    let init = if parser.check(&crate::lexer::Token::Semicolon) {
        None
    } else {
        Some(Box::new(parse_statement(parser)?))
    };
    
    let condition = if parser.check(&crate::lexer::Token::Semicolon) {
        None
    } else {
        Some(parse_expression(parser)?)
    };
    parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after for condition")?;
    
    let update = if parser.check(&crate::lexer::Token::RParen) {
        None
    } else {
        Some(parse_expression(parser)?)
    };
    
    parser.consume(&crate::lexer::Token::RParen, "Expected ')' after for clauses")?;
    
    let body = Box::new(parse_statement(parser)?);
    
    Ok(Stmt::For(ForStmt {
        init,
        condition,
        update,
        body,
        label: None,
        loc,
    }))
}

/// 解析 do-while 语句
pub fn parse_do_while_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'do'
    
    let body = Box::new(parse_statement(parser)?);
    
    parser.consume(&crate::lexer::Token::While, "Expected 'while' after 'do'")?;
    parser.consume(&crate::lexer::Token::LParen, "Expected '(' after 'while'")?;
    let condition = parse_expression(parser)?;
    parser.consume(&crate::lexer::Token::RParen, "Expected ')' after condition")?;
    parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after do-while")?;
    
    Ok(Stmt::DoWhile(DoWhileStmt {
        condition,
        body,
        label: None,
        loc,
    }))
}

/// 解析 switch 语句
pub fn parse_switch_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'switch'
    
    parser.consume(&crate::lexer::Token::LParen, "Expected '(' after 'switch'")?;
    let expr = parse_expression(parser)?;
    parser.consume(&crate::lexer::Token::RParen, "Expected ')' after switch expression")?;
    
    parser.consume(&crate::lexer::Token::LBrace, "Expected '{' to start switch body")?;
    
    let mut cases = Vec::new();
    let mut default = None;
    
    while !parser.check(&crate::lexer::Token::RBrace) && !parser.is_at_end() {
        if parser.match_token(&crate::lexer::Token::Case) {
            // 解析 case 值
            let value = match *parser.current_token() {
                crate::lexer::Token::IntegerLiteral(Some((v, _))) => {
                    let val = v;  // v 是 i64
                    parser.advance();
                    val
                }
                _ => return Err(parser.error("Expected integer literal in case")),
            };
            parser.consume(&crate::lexer::Token::Colon, "Expected ':' after case value")?;
            
            // 解析 case 体（直到遇到另一个 case、default 或 }）
            let mut body = Vec::new();
            while !parser.check(&crate::lexer::Token::Case) && !parser.check(&crate::lexer::Token::Default)
                && !parser.check(&crate::lexer::Token::RBrace) && !parser.is_at_end() {
                body.push(parse_statement(parser)?);
            }
            
            cases.push(Case { value, body });
        } else if parser.match_token(&crate::lexer::Token::Default) {
            parser.consume(&crate::lexer::Token::Colon, "Expected ':' after 'default'")?;
            
            // 解析 default 体
            let mut body = Vec::new();
            while !parser.check(&crate::lexer::Token::Case) && !parser.check(&crate::lexer::Token::Default)
                && !parser.check(&crate::lexer::Token::RBrace) && !parser.is_at_end() {
                body.push(parse_statement(parser)?);
            }
            
            default = Some(body);
        } else {
            return Err(parser.error("Expected 'case' or 'default' in switch"));
        }
    }
    
    parser.consume(&crate::lexer::Token::RBrace, "Expected '}' to end switch body")?;
    
    Ok(Stmt::Switch(SwitchStmt {
        expr,
        cases,
        default,
        loc,
    }))
}

/// 解析 return 语句
pub fn parse_return_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let _loc = parser.current_loc();
    parser.advance(); // consume 'return'
    
    let value = if !parser.check(&crate::lexer::Token::Semicolon) {
        Some(parse_expression(parser)?)
    } else {
        None
    };
    
    parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after return")?;
    
    Ok(Stmt::Return(value))
}

/// 解析表达式语句
pub fn parse_expression_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let expr = parse_expression(parser)?;
    parser.consume(&crate::lexer::Token::Semicolon, "Expected ';' after expression")?;
    Ok(Stmt::Expr(expr))
}

/// 0.5.0.0: 解析 scope 语句
/// scope 语句创建一个栈作用域，内部声明的变量在 scope 结束时自动释放
///
/// 语法: scope { statements... }
///
/// 示例:
///   scope {
///       int x = 10;
///       println("x = " + x);
///   } // x 在这里自动释放
pub fn parse_scope_statement(parser: &mut Parser) -> cayResult<Stmt> {
    let loc = parser.current_loc();
    parser.advance(); // consume 'scope'
    
    // 解析 scope 体（代码块）
    let body = parse_block(parser)?;
    
    Ok(Stmt::Scope(ScopeStmt { body, loc }))
}
