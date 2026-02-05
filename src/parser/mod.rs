use crate::lexer::{Token, TokenWithLocation};
use crate::ast::*;
use crate::types::{Type, ParameterInfo};
use crate::error::{EolResult, EolError, parser_error, SourceLocation};

pub struct Parser {
    tokens: Vec<TokenWithLocation>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithLocation>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> EolResult<Program> {
        let mut classes = Vec::new();
        
        while !self.is_at_end() {
            if self.check(&Token::Class) || self.check(&Token::Public) {
                classes.push(self.parse_class()?);
            } else {
                return Err(self.error("Expected class declaration"));
            }
        }
        
        Ok(Program { classes })
    }

    fn parse_class(&mut self) -> EolResult<ClassDecl> {
        let loc = self.current_loc();
        let modifiers = self.parse_modifiers()?;
        
        self.consume(&Token::Class, "Expected 'class' keyword")?;
        
        let name = self.consume_identifier("Expected class name")?;
        
        let parent = if self.match_token(&Token::Colon) {
            Some(self.consume_identifier("Expected parent class name")?)
        } else {
            None
        };
        
        self.consume(&Token::LBrace, "Expected '{' after class declaration")?;
        
        let mut members = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            members.push(self.parse_class_member()?);
        }
        
        self.consume(&Token::RBrace, "Expected '}' after class body")?;
        
        Ok(ClassDecl {
            name,
            modifiers,
            parent,
            members,
            loc,
        })
    }

    fn parse_class_member(&mut self) -> EolResult<ClassMember> {
        // 向前看判断是字段或方法
        let checkpoint = self.pos;
        let _modifiers = self.parse_modifiers()?;
        
        // 如果是void，一定是方法返回类型
        if self.check(&Token::Void) {
            self.pos = checkpoint;
            return Ok(ClassMember::Method(self.parse_method()?));
        }
        
        // 如果是类型关键字，可能是字段或方法
        if self.is_type_token() {
            // 读取类型
            let _type = self.parse_type()?;
            let _name = self.consume_identifier("Expected member name")?;
            
            if self.check(&Token::LParen) {
                // 是方法
                self.pos = checkpoint;
                Ok(ClassMember::Method(self.parse_method()?))
            } else {
                // 是字段
                self.pos = checkpoint;
                Ok(ClassMember::Field(self.parse_field()?))
            }
        } else {
            Err(self.error("Expected field or method declaration"))
        }
    }

    fn parse_field(&mut self) -> EolResult<FieldDecl> {
        let loc = self.current_loc();
        let modifiers = self.parse_modifiers()?;
        let field_type = self.parse_type()?;
        let name = self.consume_identifier("Expected field name")?;
        
        let initializer = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after field declaration")?;
        
        Ok(FieldDecl {
            name,
            field_type,
            modifiers,
            initializer,
            loc,
        })
    }

    fn parse_method(&mut self) -> EolResult<MethodDecl> {
        let loc = self.current_loc();
        let modifiers = self.parse_modifiers()?;
        
        let return_type = if self.check(&Token::Void) {
            self.advance();
            Type::Void
        } else {
            self.parse_type()?
        };
        
        let name = self.consume_identifier("Expected method name")?;
        
        self.consume(&Token::LParen, "Expected '(' after method name")?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen, "Expected ')' after parameters")?;
        
        // 检查是否是native方法
        let is_native = modifiers.contains(&Modifier::Native);
        
        let body = if is_native {
            self.consume(&Token::Semicolon, "Expected ';' after native method declaration")?;
            None
        } else {
            Some(self.parse_block()?)
        };
        
        Ok(MethodDecl {
            name,
            modifiers,
            return_type,
            params,
            body,
            loc,
        })
    }

    fn parse_modifiers(&mut self) -> EolResult<Vec<Modifier>> {
        let mut modifiers = Vec::new();
        
        loop {
            match self.current_token() {
                Token::Public => {
                    modifiers.push(Modifier::Public);
                    self.advance();
                }
                Token::Private => {
                    modifiers.push(Modifier::Private);
                    self.advance();
                }
                Token::Protected => {
                    modifiers.push(Modifier::Protected);
                    self.advance();
                }
                Token::Static => {
                    modifiers.push(Modifier::Static);
                    self.advance();
                }
                Token::Final => {
                    modifiers.push(Modifier::Final);
                    self.advance();
                }
                Token::Abstract => {
                    modifiers.push(Modifier::Abstract);
                    self.advance();
                }
                Token::Native => {
                    modifiers.push(Modifier::Native);
                    self.advance();
                }
                _ => break,
            }
        }
        
        Ok(modifiers)
    }

    fn parse_parameters(&mut self) -> EolResult<Vec<ParameterInfo>> {
        let mut params = Vec::new();
        
        if !self.check(&Token::RParen) {
            loop {
                let param_type = self.parse_type()?;
                let name = self.consume_identifier("Expected parameter name")?;
                
                params.push(ParameterInfo {
                    name,
                    param_type,
                });
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        Ok(params)
    }

    fn parse_type(&mut self) -> EolResult<Type> {
        let base_type = match self.current_token() {
            Token::Int => { self.advance(); Type::Int32 }
            Token::Long => { self.advance(); Type::Int64 }
            Token::Float => { self.advance(); Type::Float32 }
            Token::Double => { self.advance(); Type::Float64 }
            Token::Bool => { self.advance(); Type::Bool }
            Token::String => { self.advance(); Type::String }
            Token::Char => { self.advance(); Type::Char }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Type::Object(name)
            }
            _ => return Err(self.error("Expected type")),
        };
        
        // 检查数组类型
        if self.match_token(&Token::LBracket) {
            self.consume(&Token::RBracket, "Expected ']' after '['")?;
            Ok(Type::Array(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    fn parse_block(&mut self) -> EolResult<Block> {
        let loc = self.current_loc();
        self.consume(&Token::LBrace, "Expected '{' to start block")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&Token::RBrace, "Expected '}' to end block")?;
        
        Ok(Block { statements, loc })
    }

    fn parse_statement(&mut self) -> EolResult<Stmt> {
        match self.current_token() {
            Token::LBrace => Ok(Stmt::Block(self.parse_block()?)),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                let _loc = self.current_loc();
                self.advance();
                self.consume(&Token::Semicolon, "Expected ';' after break")?;
                Ok(Stmt::Break)
            }
            Token::Continue => {
                let _loc = self.current_loc();
                self.advance();
                self.consume(&Token::Semicolon, "Expected ';' after continue")?;
                Ok(Stmt::Continue)
            }
            _ => {
                // 检查是否是变量声明（只能是原始类型关键字，不能是任意标识符）
                if self.is_primitive_type_token() || self.check(&Token::Final) {
                    self.parse_var_decl()
                } else {
                    self.parse_expression_statement()
                }
            }
        }
    }

    fn parse_var_decl(&mut self) -> EolResult<Stmt> {
        let loc = self.current_loc();
        
        let is_final = self.match_token(&Token::Final);
        
        let var_type = self.parse_type()?;
        let name = self.consume_identifier("Expected variable name")?;
        
        let initializer = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after variable declaration")?;
        
        Ok(Stmt::VarDecl(VarDecl {
            name,
            var_type,
            initializer,
            is_final,
            loc,
        }))
    }

    fn parse_if_statement(&mut self) -> EolResult<Stmt> {
        let loc = self.current_loc();
        self.advance(); // consume 'if'
        
        self.consume(&Token::LParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen, "Expected ')' after if condition")?;
        
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.match_token(&Token::Else) {
            Some(Box::new(self.parse_statement()?))
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

    fn parse_while_statement(&mut self) -> EolResult<Stmt> {
        let loc = self.current_loc();
        self.advance(); // consume 'while'
        
        self.consume(&Token::LParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RParen, "Expected ')' after while condition")?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(Stmt::While(WhileStmt {
            condition,
            body,
            loc,
        }))
    }

    fn parse_for_statement(&mut self) -> EolResult<Stmt> {
        let loc = self.current_loc();
        self.advance(); // consume 'for'
        
        self.consume(&Token::LParen, "Expected '(' after 'for'")?;
        
        let init = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };
        
        let condition = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&Token::Semicolon, "Expected ';' after for condition")?;
        
        let update = if self.check(&Token::RParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.consume(&Token::RParen, "Expected ')' after for clauses")?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(Stmt::For(ForStmt {
            init,
            condition,
            update,
            body,
            loc,
        }))
    }

    fn parse_return_statement(&mut self) -> EolResult<Stmt> {
        let _loc = self.current_loc();
        self.advance(); // consume 'return'
        
        let value = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expected ';' after return")?;
        
        Ok(Stmt::Return(value))
    }

    fn parse_expression_statement(&mut self) -> EolResult<Stmt> {
        let expr = self.parse_expression()?;
        self.consume(&Token::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expression(&mut self) -> EolResult<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> EolResult<Expr> {
        let loc = self.current_loc();
        let expr = self.parse_or()?;
        
        if let Some(op) = self.match_assignment_op() {
            let value = self.parse_assignment()?;
            return Ok(Expr::Assignment(AssignmentExpr {
                target: Box::new(expr),
                value: Box::new(value),
                op,
                loc,
            }));
        }
        
        Ok(expr)
    }

    fn parse_or(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_and()?;
        
        while self.match_token(&Token::OrOr) {
            let loc = self.current_loc();
            let right = self.parse_and()?;
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
                loc,
            });
        }
        
        Ok(left)
    }

    fn parse_and(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_bitwise_or()?;
        
        while self.match_token(&Token::AndAnd) {
            let loc = self.current_loc();
            let right = self.parse_bitwise_or()?;
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                loc,
            });
        }
        
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_bitwise_xor()?;
        
        while self.match_token(&Token::Pipe) {
            let loc = self.current_loc();
            let right = self.parse_bitwise_xor()?;
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitOr,
                right: Box::new(right),
                loc,
            });
        }
        
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_bitwise_and()?;
        
        while self.match_token(&Token::Caret) {
            let loc = self.current_loc();
            let right = self.parse_bitwise_and()?;
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitXor,
                right: Box::new(right),
                loc,
            });
        }
        
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(&Token::Ampersand) {
            let loc = self.current_loc();
            let right = self.parse_equality()?;
            left = Expr::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
                loc,
            });
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_comparison()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::EqEq) {
                let right = self.parse_comparison()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Eq,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::NotEq) {
                let right = self.parse_comparison()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Ne,
                    right: Box::new(right),
                    loc,
                });
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_shift()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::Lt) {
                let right = self.parse_shift()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Lt,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Le) {
                let right = self.parse_shift()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Le,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Gt) {
                let right = self.parse_shift()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Gt,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Ge) {
                let right = self.parse_shift()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Ge,
                    right: Box::new(right),
                    loc,
                });
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_shift(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_term()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::Shl) {
                let right = self.parse_term()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Shl,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Shr) {
                let right = self.parse_term()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Shr,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::UnsignedShr) {
                let right = self.parse_term()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::UnsignedShr,
                    right: Box::new(right),
                    loc,
                });
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_term(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_factor()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::Plus) {
                let right = self.parse_factor()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Add,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Minus) {
                let right = self.parse_factor()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Sub,
                    right: Box::new(right),
                    loc,
                });
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_factor(&mut self) -> EolResult<Expr> {
        let mut left = self.parse_unary()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::Star) {
                let right = self.parse_unary()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Mul,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Slash) {
                let right = self.parse_unary()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Div,
                    right: Box::new(right),
                    loc,
                });
            } else if self.match_token(&Token::Percent) {
                let right = self.parse_unary()?;
                left = Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: BinaryOp::Mod,
                    right: Box::new(right),
                    loc,
                });
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> EolResult<Expr> {
        let loc = self.current_loc();
        
        if self.match_token(&Token::Minus) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
                loc,
            }));
        }
        
        if self.match_token(&Token::Bang) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                loc,
            }));
        }
        
        if self.match_token(&Token::Tilde) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op: UnaryOp::BitNot,
                operand: Box::new(operand),
                loc,
            }));
        }
        
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> EolResult<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            let loc = self.current_loc();
            if self.match_token(&Token::LParen) {
                // 函数调用
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen, "Expected ')' after arguments")?;
                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                    loc,
                });
            } else if self.match_token(&Token::Dot) {
                // 成员访问
                let member = self.consume_identifier("Expected member name after '.'")?;
                expr = Expr::MemberAccess(MemberAccessExpr {
                    object: Box::new(expr),
                    member,
                    loc,
                });
            } else if self.match_token(&Token::LBracket) {
                // 数组索引
                let _index = self.parse_expression()?;
                self.consume(&Token::RBracket, "Expected ']' after index")?;
                // TODO: 数组索引作为特殊的成员访问
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> EolResult<Expr> {
        let loc = self.current_loc();
        
        match self.current_token() {
            Token::IntegerLiteral(Some(val)) => {
                let val = *val;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Int(val)))
            }
            Token::FloatLiteral(Some(val)) => {
                let val = *val;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Float(val)))
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(LiteralValue::String(s)))
            }
            Token::CharLiteral(Some(c)) => {
                let c = *c;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Char(c)))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(false)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Null))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::New => {
                self.advance();
                let class_name = self.consume_identifier("Expected class name after 'new'")?;
                self.consume(&Token::LParen, "Expected '(' after class name")?;
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen, "Expected ')' after arguments")?;
                Ok(Expr::New(NewExpr {
                    class_name,
                    args,
                    loc,
                }))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(self.error("Expected expression")),
        }
    }

    fn parse_arguments(&mut self) -> EolResult<Vec<Expr>> {
        let mut args = Vec::new();
        
        if !self.check(&Token::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        Ok(args)
    }

    fn match_assignment_op(&mut self) -> Option<AssignOp> {
        if self.check(&Token::Assign) {
            self.advance();
            Some(AssignOp::Assign)
        } else if self.check(&Token::AddAssign) {
            self.advance();
            Some(AssignOp::AddAssign)
        } else if self.check(&Token::SubAssign) {
            self.advance();
            Some(AssignOp::SubAssign)
        } else if self.check(&Token::MulAssign) {
            self.advance();
            Some(AssignOp::MulAssign)
        } else if self.check(&Token::DivAssign) {
            self.advance();
            Some(AssignOp::DivAssign)
        } else if self.check(&Token::ModAssign) {
            self.advance();
            Some(AssignOp::ModAssign)
        } else {
            None
        }
    }

    // 辅助方法
    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() - 1
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn current_loc(&self) -> SourceLocation {
        self.tokens[self.pos].loc.clone()
    }

    fn previous_loc(&self) -> SourceLocation {
        if self.pos > 0 {
            self.tokens[self.pos - 1].loc.clone()
        } else {
            self.tokens[0].loc.clone()
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1].token
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.current_token() == token
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token: &Token, message: &str) -> EolResult<&Token> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            // 如果期望分号但没找到，使用上一个token的位置
            let loc = if message.contains("';'") {
                self.previous_loc()
            } else {
                self.current_loc()
            };
            Err(crate::error::parser_error(loc.line, loc.column, message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> EolResult<String> {
        if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }

    fn is_type_token(&self) -> bool {
        matches!(self.current_token(),
            Token::Int | Token::Long | Token::Float |
            Token::Double | Token::Bool | Token::String |
            Token::Char | Token::Identifier(_)
        )
    }

    fn is_primitive_type_token(&self) -> bool {
        matches!(self.current_token(),
            Token::Int | Token::Long | Token::Float |
            Token::Double | Token::Bool | Token::String |
            Token::Char
        )
    }

    fn error(&self, message: &str) -> EolError {
        let loc = &self.tokens[self.pos].loc;
        parser_error(loc.line, loc.column, message)
    }
}

pub fn parse(tokens: Vec<TokenWithLocation>) -> EolResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
