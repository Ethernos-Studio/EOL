//! cay 语法分析器
//!
//! 本模块将词法分析器生成的令牌流解析为抽象语法树 (AST)。
//! 已重构为多个子模块以提高可维护性。

mod classes;
mod types;
mod statements;
mod expressions;
mod utils;

use crate::lexer::TokenWithLocation;
use crate::ast::Program;
use crate::error::cayResult;

/// 语法分析器
pub struct Parser {
    /// 令牌流
    pub tokens: Vec<TokenWithLocation>,
    /// 当前解析位置
    pub pos: usize,
}

impl Parser {
    /// 创建新的语法分析器
    pub fn new(tokens: Vec<TokenWithLocation>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析整个程序
    pub fn parse(&mut self) -> cayResult<Program> {
        let mut classes = Vec::new();
        
        while !self.is_at_end() {
            if self.check(&crate::lexer::Token::Class)
                || self.check(&crate::lexer::Token::Public)
                || self.check(&crate::lexer::Token::Private)
                || self.check(&crate::lexer::Token::Protected)
                || self.check(&crate::lexer::Token::AtMain)
            {
                classes.push(self.parse_class()?);
            } else {
                return Err(self.error("Expected class declaration"));
            }
        }
        
        Ok(Program { classes })
    }

    // 类解析方法
    fn parse_class(&mut self) -> cayResult<crate::ast::ClassDecl> {
        classes::parse_class(self)
    }
    
    fn parse_class_member(&mut self) -> cayResult<crate::ast::ClassMember> {
        classes::parse_class_member(self)
    }
    
    fn parse_field(&mut self) -> cayResult<crate::ast::FieldDecl> {
        classes::parse_field(self)
    }
    
    fn parse_method(&mut self) -> cayResult<crate::ast::MethodDecl> {
        classes::parse_method(self)
    }
    
    fn parse_modifiers(&mut self) -> cayResult<Vec<crate::ast::Modifier>> {
        classes::parse_modifiers(self)
    }
    
    fn parse_parameters(&mut self) -> cayResult<Vec<crate::types::ParameterInfo>> {
        classes::parse_parameters(self)
    }
    
    // 类型解析方法
    fn parse_type(&mut self) -> cayResult<crate::types::Type> {
        types::parse_type(self)
    }
    
    fn is_type_token(&self) -> bool {
        types::is_type_token(self)
    }
    
    fn is_primitive_type_token(&self) -> bool {
        types::is_primitive_type_token(self)
    }
    
    // 语句解析方法
    fn parse_block(&mut self) -> cayResult<crate::ast::Block> {
        statements::parse_block(self)
    }
    
    fn parse_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_statement(self)
    }
    
    fn parse_var_decl(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_var_decl(self)
    }
    
    fn parse_if_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_if_statement(self)
    }
    
    fn parse_while_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_while_statement(self)
    }
    
    fn parse_for_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_for_statement(self)
    }
    
    fn parse_do_while_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_do_while_statement(self)
    }
    
    fn parse_switch_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_switch_statement(self)
    }
    
    fn parse_return_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_return_statement(self)
    }
    
    fn parse_expression_statement(&mut self) -> cayResult<crate::ast::Stmt> {
        statements::parse_expression_statement(self)
    }
    
    // 表达式解析方法
    fn parse_expression(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_expression(self)
    }
    
    fn parse_assignment(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_assignment(self)
    }
    
    fn parse_or(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_or(self)
    }
    
    fn parse_and(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_and(self)
    }
    
    fn parse_bitwise_or(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_bitwise_or(self)
    }
    
    fn parse_bitwise_xor(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_bitwise_xor(self)
    }
    
    fn parse_bitwise_and(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_bitwise_and(self)
    }
    
    fn parse_equality(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_equality(self)
    }
    
    fn parse_comparison(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_comparison(self)
    }
    
    fn parse_shift(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_shift(self)
    }
    
    fn parse_term(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_term(self)
    }
    
    fn parse_factor(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_factor(self)
    }
    
    fn parse_unary(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_unary(self)
    }
    
    fn parse_postfix(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_postfix(self)
    }
    
    fn parse_primary(&mut self) -> cayResult<crate::ast::Expr> {
        expressions::parse_primary(self)
    }
    
    fn parse_arguments(&mut self) -> cayResult<Vec<crate::ast::Expr>> {
        expressions::parse_arguments(self)
    }
    
    fn match_assignment_op(&mut self) -> Option<crate::ast::AssignOp> {
        expressions::match_assignment_op(self)
    }
    
    // 辅助方法
    fn is_at_end(&self) -> bool {
        utils::is_at_end(self)
    }
    
    fn current_token(&self) -> &crate::lexer::Token {
        utils::current_token(self)
    }
    
    fn current_loc(&self) -> crate::error::SourceLocation {
        utils::current_loc(self)
    }
    
    fn previous_loc(&self) -> crate::error::SourceLocation {
        utils::previous_loc(self)
    }
    
    fn advance(&mut self) -> &crate::lexer::Token {
        utils::advance(self)
    }
    
    fn check(&self, token: &crate::lexer::Token) -> bool {
        utils::check(self, token)
    }
    
    fn match_token(&mut self, token: &crate::lexer::Token) -> bool {
        utils::match_token(self, token)
    }
    
    fn consume(&mut self, token: &crate::lexer::Token, message: &str) -> cayResult<&crate::lexer::Token> {
        utils::consume(self, token, message)
    }
    
    fn consume_identifier(&mut self, message: &str) -> cayResult<String> {
        utils::consume_identifier(self, message)
    }
    
    fn error(&self, message: &str) -> crate::error::cayError {
        utils::error(self, message)
    }
}

/// 解析令牌流生成 AST
pub fn parse(tokens: Vec<TokenWithLocation>) -> cayResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
