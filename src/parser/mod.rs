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
        let mut interfaces = Vec::new();
        let mut top_level_functions = Vec::new();

        while !self.is_at_end() {
            if self.check(&crate::lexer::Token::Interface)
                || (self.check(&crate::lexer::Token::Public) && self.check_next(&crate::lexer::Token::Interface))
            {
                interfaces.push(self.parse_interface()?);
            } else if self.check(&crate::lexer::Token::Class)
                || self.check(&crate::lexer::Token::Private)
                || self.check(&crate::lexer::Token::Protected)
                || self.check(&crate::lexer::Token::AtMain)
            {
                classes.push(self.parse_class()?);
            } else if self.check(&crate::lexer::Token::Public) {
                // 检查是否是顶层 main 函数: public int main() 或 public int main(String[] args)
                if self.check_top_level_main() {
                    top_level_functions.push(self.parse_top_level_function()?);
                } else {
                    // 否则可能是 public class
                    classes.push(self.parse_class()?);
                }
            } else {
                return Err(self.error("Expected class, interface, or top-level function declaration"));
            }
        }

        Ok(Program { classes, interfaces, top_level_functions })
    }

    // 类解析方法
    fn parse_class(&mut self) -> cayResult<crate::ast::ClassDecl> {
        classes::parse_class(self)
    }

    fn parse_interface(&mut self) -> cayResult<crate::ast::InterfaceDecl> {
        classes::parse_interface(self)
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

    fn check_next(&self, token: &crate::lexer::Token) -> bool {
        utils::check_next(self, token)
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

    /// 检查是否是顶层 main 函数
    fn check_top_level_main(&self) -> bool {
        // 需要 lookahead: public (int|void) main
        let mut pos = self.pos;
        // 跳过 public
        if pos >= self.tokens.len() {
            return false;
        }
        pos += 1;

        // 检查是否是 int 或 void
        if pos >= self.tokens.len() {
            return false;
        }
        match &self.tokens[pos].token {
            crate::lexer::Token::Int | crate::lexer::Token::Void => {}
            _ => return false,
        }
        pos += 1;

        // 检查是否是 main
        if pos >= self.tokens.len() {
            return false;
        }
        match &self.tokens[pos].token {
            crate::lexer::Token::Identifier(name) if name == "main" => true,
            _ => false,
        }
    }

    /// 解析顶层函数
    fn parse_top_level_function(&mut self) -> cayResult<crate::ast::TopLevelFunction> {
        let loc = self.current_loc();

        // 必须是以 public 开始
        self.consume(&crate::lexer::Token::Public, "Expected 'public' for top-level function")?;

        // 解析返回类型
        let return_type = match self.current_token() {
            crate::lexer::Token::Int => { self.advance(); crate::types::Type::Int32 }
            crate::lexer::Token::Void => { self.advance(); crate::types::Type::Void }
            _ => return Err(self.error("Top-level main function must return int or void")),
        };

        // 解析函数名
        let name = self.consume_identifier("Expected function name")?;

        // 解析参数列表
        self.consume(&crate::lexer::Token::LParen, "Expected '(' after function name")?;
        let params = self.parse_parameters()?;
        self.consume(&crate::lexer::Token::RParen, "Expected ')' after parameters")?;

        // 解析函数体
        let body = self.parse_block()?;

        Ok(crate::ast::TopLevelFunction {
            name,
            modifiers: vec![crate::ast::Modifier::Public],
            return_type,
            params,
            body,
            loc,
        })
    }
}

/// 解析令牌流生成 AST
pub fn parse(tokens: Vec<TokenWithLocation>) -> cayResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
