//! 类相关解析

use crate::ast::*;
use crate::types::{Type, ParameterInfo, InterfaceInfo};
use crate::error::cayResult;
use crate::lexer::Token;
use crate::error::SourceLocation;
use super::Parser;
use super::types::{parse_type, is_type_token};
use super::expressions::parse_expression;
use super::statements::{parse_block, parse_statement};

/// 解析类声明
pub fn parse_class(parser: &mut Parser) -> cayResult<ClassDecl> {
    let loc = parser.current_loc();

    // 解析所有修饰符（包括 @main 注解）
    let modifiers = parse_modifiers(parser)?;

    parser.consume(&Token::Class, "Expected 'class' keyword")?;

    let name = parser.consume_identifier("Expected class name")?;

    // 支持 extends 关键字或 : 符号作为继承语法
    let parent = if parser.match_token(&Token::Extends) {
        Some(parser.consume_identifier("Expected parent class name after 'extends'")?)
    } else if parser.match_token(&Token::Colon) {
        // 保留 : 符号作为兼容语法
        Some(parser.consume_identifier("Expected parent class name after ':'")?)
    } else {
        None
    };

    // 解析实现的接口
    let mut interfaces = Vec::new();
    if parser.match_token(&Token::Implements) {
        loop {
            let interface_name = parser.consume_identifier("Expected interface name")?;
            interfaces.push(interface_name);
            if !parser.match_token(&Token::Comma) {
                break;
            }
        }
    }

    parser.consume(&Token::LBrace, "Expected '{' after class declaration")?;

    let mut members = Vec::new();
    while !parser.check(&Token::RBrace) && !parser.is_at_end() {
        members.push(parse_class_member(parser)?);
    }

    parser.consume(&Token::RBrace, "Expected '}' after class body")?;

    Ok(ClassDecl {
        name,
        modifiers,
        parent,
        interfaces,
        members,
        loc,
    })
}

/// 解析接口声明
pub fn parse_interface(parser: &mut Parser) -> cayResult<InterfaceDecl> {
    let loc = parser.current_loc();

    // 解析修饰符
    let modifiers = parse_modifiers(parser)?;

    parser.consume(&Token::Interface, "Expected 'interface' keyword")?;

    let name = parser.consume_identifier("Expected interface name")?;

    parser.consume(&Token::LBrace, "Expected '{' after interface declaration")?;

    // 接口只能包含方法声明（没有方法体）
    let mut methods = Vec::new();
    while !parser.check(&Token::RBrace) && !parser.is_at_end() {
        methods.push(parse_interface_method(parser)?);
    }

    parser.consume(&Token::RBrace, "Expected '}' after interface body")?;

    Ok(InterfaceDecl {
        name,
        modifiers,
        methods,
        loc,
    })
}

/// 解析接口方法（只有声明，没有实现）
fn parse_interface_method(parser: &mut Parser) -> cayResult<MethodDecl> {
    let loc = parser.current_loc();
    let modifiers = parse_modifiers(parser)?;

    let return_type = if parser.check(&Token::Void) {
        parser.advance();
        Type::Void
    } else {
        parse_type(parser)?
    };

    let name = parser.consume_identifier("Expected method name")?;

    parser.consume(&Token::LParen, "Expected '(' after method name")?;
    let params = parse_parameters(parser)?;
    parser.consume(&Token::RParen, "Expected ')' after parameters")?;

    // 接口方法必须以分号结束，没有方法体
    parser.consume(&Token::Semicolon, "Expected ';' after interface method declaration")?;

    Ok(MethodDecl {
        name,
        modifiers,
        return_type,
        params,
        body: None,  // 接口方法没有方法体
        loc,
    })
}

/// 解析类成员（字段、方法、构造函数、析构函数或初始化块）
pub fn parse_class_member(parser: &mut Parser) -> cayResult<ClassMember> {
    // 向前看判断成员类型
    let checkpoint = parser.pos;
    let modifiers = parse_modifiers(parser)?;
    
    // 检查是否是静态初始化块 static { ... }
    if modifiers.contains(&Modifier::Static) && parser.check(&Token::LBrace) {
        parser.pos = checkpoint;
        return Ok(ClassMember::StaticInitializer(parse_static_initializer(parser)?));
    }
    
    // 检查是否是初始化块 { ... }
    if parser.check(&Token::LBrace) {
        parser.pos = checkpoint;
        return Ok(ClassMember::InstanceInitializer(parse_instance_initializer(parser)?));
    }
    
    // 检查是否是析构函数 ~ClassName() { ... }
    if parser.check(&Token::Tilde) {
        parser.pos = checkpoint;
        return Ok(ClassMember::Destructor(parse_destructor(parser)?));
    }
    
    // 如果是void，一定是方法返回类型
    if parser.check(&Token::Void) {
        parser.pos = checkpoint;
        return Ok(ClassMember::Method(parse_method(parser)?));
    }
    
    // 检查是否是构造函数：类名(...)
    // 构造函数的特征是：标识符后直接跟'('，且不是类型关键字（void, int等）
    if matches!(parser.current_token(), Token::Identifier(_)) {
        // 向前看：检查下一个token是否是 '('
        let current_pos = parser.pos;
        parser.advance(); // 跳过标识符
        
        if parser.check(&Token::LParen) {
            // 是构造函数 - 回溯到checkpoint并解析
            parser.pos = checkpoint;
            
            // 直接解析构造函数
            let loc = parser.current_loc();
            let ctor_modifiers = parse_modifiers(parser)?;
            let _ctor_name = parser.consume_identifier("Expected constructor name")?;
            
            parser.consume(&Token::LParen, "Expected '(' after constructor name")?;
            let ctor_params = parse_parameters(parser)?;
            parser.consume(&Token::RParen, "Expected ')' after constructor parameters")?;
            
            // 解析构造链调用 this() 或 super()
            let ctor_call_result = parse_constructor_call(parser)?;
            let constructor_call = ctor_call_result.call;
            
            // 解析构造函数体
            // 如果 Java 风格的构造链调用已经消耗了 {，则不需要再解析 {
            let ctor_body = if ctor_call_result.consumed_lbrace {
                // 已经消耗了 {，直接解析语句直到 }
                let mut statements = Vec::new();
                while !parser.check(&Token::RBrace) && !parser.is_at_end() {
                    statements.push(parse_statement(parser)?);
                }
                parser.consume(&Token::RBrace, "Expected '}' after constructor body")?;
                Block { statements, loc: parser.current_loc() }
            } else {
                parse_block(parser)?
            };
            
            return Ok(ClassMember::Constructor(ConstructorDecl {
                modifiers: ctor_modifiers,
                params: ctor_params,
                body: ctor_body,
                constructor_call,
                loc,
            }));
        } else {
            // 不是构造函数，回退位置
            parser.pos = current_pos;
        }
    }
    
    // 如果是类型关键字，可能是字段或方法
    if is_type_token(parser) {
        // 读取类型
        let member_type = parse_type(parser)?;
        let member_name = parser.consume_identifier("Expected member name")?;
        
        if parser.check(&Token::LParen) {
            // 是方法
            parser.pos = checkpoint;
            Ok(ClassMember::Method(parse_method(parser)?))
        } else {
            // 是字段
            parser.pos = checkpoint;
            Ok(ClassMember::Field(parse_field(parser)?))
        }
    } else {
        Err(parser.error("Expected field, method, constructor, or destructor declaration"))
    }
}

/// 解析字段声明
pub fn parse_field(parser: &mut Parser) -> cayResult<FieldDecl> {
    let loc = parser.current_loc();
    let modifiers = parse_modifiers(parser)?;
    let field_type = parse_type(parser)?;
    let name = parser.consume_identifier("Expected field name")?;
    
    let initializer = if parser.match_token(&Token::Assign) {
        Some(parse_expression(parser)?)
    } else {
        None
    };
    
    parser.consume(&Token::Semicolon, "Expected ';' after field declaration")?;
    
    Ok(FieldDecl {
        name,
        field_type,
        modifiers,
        initializer,
        loc,
    })
}

/// 解析方法声明
pub fn parse_method(parser: &mut Parser) -> cayResult<MethodDecl> {
    let loc = parser.current_loc();
    let modifiers = parse_modifiers(parser)?;
    
    let return_type = if parser.check(&Token::Void) {
        parser.advance();
        Type::Void
    } else {
        parse_type(parser)?
    };
    
    let name = parser.consume_identifier("Expected method name")?;
    
    parser.consume(&Token::LParen, "Expected '(' after method name")?;
    let params = parse_parameters(parser)?;
    parser.consume(&Token::RParen, "Expected ')' after parameters")?;
    
    // 检查是否是native方法或abstract方法（这两种都可以没有方法体）
    let is_native = modifiers.contains(&Modifier::Native);
    let is_abstract = modifiers.contains(&Modifier::Abstract);
    
    let body = if is_native || is_abstract {
        parser.consume(&Token::Semicolon, "Expected ';' after method declaration")?;
        None
    } else {
        Some(parse_block(parser)?)
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

/// 解析构造函数声明
/// 格式: [modifiers] ClassName([params]) [throws ...] { body }
/// 或: [modifiers] ClassName([params]) : this(args) { body }
/// 或: [modifiers] ClassName([params]) : super(args) { body }
pub fn parse_constructor(parser: &mut Parser) -> cayResult<ConstructorDecl> {
    let loc = parser.current_loc();
    let modifiers = parse_modifiers(parser)?;
    
    // 构造函数名（必须与类名相同）
    let _name = parser.consume_identifier("Expected constructor name")?;
    
    parser.consume(&Token::LParen, "Expected '(' after constructor name")?;
    let params = parse_parameters(parser)?;
    parser.consume(&Token::RParen, "Expected ')' after constructor parameters")?;
    
    // 解析构造链调用 this() 或 super()
    let ctor_call_result = parse_constructor_call(parser)?;
    let constructor_call = ctor_call_result.call;
    
    // 解析构造函数体
    // 如果 Java 风格的构造链调用已经消耗了 {，则不需要再解析 {
    let body = if ctor_call_result.consumed_lbrace {
        // 已经消耗了 {，直接解析语句直到 }
        let mut statements = Vec::new();
        while !parser.check(&Token::RBrace) && !parser.is_at_end() {
            statements.push(parse_statement(parser)?);
        }
        parser.consume(&Token::RBrace, "Expected '}' after constructor body")?;
        Block { statements, loc: parser.current_loc() }
    } else {
        parse_block(parser)?
    };
    
    Ok(ConstructorDecl {
        modifiers,
        params,
        body,
        constructor_call,
        loc,
    })
}

/// 构造链调用解析结果
#[derive(Debug)]
struct ConstructorCallResult {
    pub call: Option<ConstructorCall>,
    pub consumed_lbrace: bool, // 是否消耗了左大括号
}

/// 解析构造链调用 this() 或 super()
/// 支持两种风格：
/// - C++风格: : this(args) 或 : super(args)（在构造函数参数列表后）
/// - Java风格: this(args) 或 super(args)（作为构造函数体的第一条语句）
fn parse_constructor_call(parser: &mut Parser) -> cayResult<ConstructorCallResult> {
    // 检查是否有冒号（C++风格）
    if parser.match_token(&Token::Colon) {
        // C++风格: : this(args) 或 : super(args)
        if parser.match_token(&Token::This) {
            parser.consume(&Token::LParen, "Expected '(' after 'this'")?;
            let args = parse_constructor_call_args(parser)?;
            parser.consume(&Token::RParen, "Expected ')' after 'this' arguments")?;
            return Ok(ConstructorCallResult {
                call: Some(ConstructorCall::This(args)),
                consumed_lbrace: false,
            });
        } else if parser.match_token(&Token::Super) {
            parser.consume(&Token::LParen, "Expected '(' after 'super'")?;
            let args = parse_constructor_call_args(parser)?;
            parser.consume(&Token::RParen, "Expected ')' after 'super' arguments")?;
            return Ok(ConstructorCallResult {
                call: Some(ConstructorCall::Super(args)),
                consumed_lbrace: false,
            });
        } else {
            return Err(parser.error("Expected 'this' or 'super' after ':'"));
        }
    }
    
    // Java风格: 检查是否是 this(args) 或 super(args) 作为第一条语句
    // 向前看：{ this( 或 { super(
    if parser.check(&Token::LBrace) {
        // 保存当前位置
        let checkpoint = parser.pos;
        parser.advance(); // 跳过 {
        
        // 检查是否是 this(
        if parser.match_token(&Token::This) {
            if parser.check(&Token::LParen) {
                parser.advance(); // 跳过 (
                let args = parse_constructor_call_args(parser)?;
                parser.consume(&Token::RParen, "Expected ')' after 'this' arguments")?;
                parser.consume(&Token::Semicolon, "Expected ';' after this() call")?;
                return Ok(ConstructorCallResult {
                    call: Some(ConstructorCall::This(args)),
                    consumed_lbrace: true,
                });
            } else {
                // 不是 this(...)，回退
                parser.pos = checkpoint;
            }
        } else if parser.match_token(&Token::Super) {
            if parser.check(&Token::LParen) {
                parser.advance(); // 跳过 (
                let args = parse_constructor_call_args(parser)?;
                parser.consume(&Token::RParen, "Expected ')' after 'super' arguments")?;
                parser.consume(&Token::Semicolon, "Expected ';' after super() call")?;
                return Ok(ConstructorCallResult {
                    call: Some(ConstructorCall::Super(args)),
                    consumed_lbrace: true,
                });
            } else {
                // 不是 super(...)，回退
                parser.pos = checkpoint;
            }
        } else {
            // 不是 this 或 super，回退
            parser.pos = checkpoint;
        }
    }
    
    Ok(ConstructorCallResult {
        call: None,
        consumed_lbrace: false,
    })
}

/// 解析构造函数调用参数
fn parse_constructor_call_args(parser: &mut Parser) -> cayResult<Vec<Expr>> {
    let mut args = Vec::new();
    
    if !parser.check(&Token::RParen) {
        loop {
            args.push(parse_expression(parser)?);
            if !parser.match_token(&Token::Comma) {
                break;
            }
        }
    }
    
    Ok(args)
}

/// 解析析构函数声明
/// 格式: ~ClassName() { body }
pub fn parse_destructor(parser: &mut Parser) -> cayResult<DestructorDecl> {
    let loc = parser.current_loc();
    let modifiers = parse_modifiers(parser)?;
    
    // 消耗 ~
    parser.consume(&Token::Tilde, "Expected '~' for destructor")?;
    
    // 析构函数名（必须与类名相同）
    let _name = parser.consume_identifier("Expected destructor name")?;
    
    parser.consume(&Token::LParen, "Expected '(' after destructor name")?;
    parser.consume(&Token::RParen, "Expected ')' after destructor parameters")?;
    
    // 解析析构函数体
    let body = parse_block(parser)?;
    
    Ok(DestructorDecl {
        modifiers,
        body,
        loc,
    })
}

/// 解析实例初始化块
/// 格式: { statements }
pub fn parse_instance_initializer(parser: &mut Parser) -> cayResult<Block> {
    parse_block(parser)
}

/// 解析静态初始化块
/// 格式: static { statements }
pub fn parse_static_initializer(parser: &mut Parser) -> cayResult<Block> {
    let _modifiers = parse_modifiers(parser)?; // 消耗 static
    parse_block(parser)
}

/// 解析修饰符列表（包括注解）
pub fn parse_modifiers(parser: &mut Parser) -> cayResult<Vec<Modifier>> {
    let mut modifiers = Vec::new();
    
    loop {
        match parser.current_token() {
            Token::Public => {
                modifiers.push(Modifier::Public);
                parser.advance();
            }
            Token::Private => {
                modifiers.push(Modifier::Private);
                parser.advance();
            }
            Token::Protected => {
                modifiers.push(Modifier::Protected);
                parser.advance();
            }
            Token::Static => {
                modifiers.push(Modifier::Static);
                parser.advance();
            }
            Token::Final => {
                modifiers.push(Modifier::Final);
                parser.advance();
            }
            Token::Abstract => {
                modifiers.push(Modifier::Abstract);
                parser.advance();
            }
            Token::Native => {
                modifiers.push(Modifier::Native);
                parser.advance();
            }
            Token::AtOverride => {
                modifiers.push(Modifier::Override);
                parser.advance();
            }
            Token::AtMain => {
                modifiers.push(Modifier::Main);
                parser.advance();
            }
            _ => break,
        }
    }
    
    Ok(modifiers)
}

/// 解析参数列表（支持可变参数）
pub fn parse_parameters(parser: &mut Parser) -> cayResult<Vec<ParameterInfo>> {
    let mut params = Vec::new();

    if !parser.check(&Token::RParen) {
        loop {
            // 检查是否是裸可变参数 ...（C 风格 extern 函数声明，如 int printf(const char* fmt, ...);）
            if parser.check(&Token::DotDotDot) {
                parser.advance(); // 消费 ...
                // 为可变参数创建一个特殊参数名
                params.push(ParameterInfo::new_varargs("...".to_string(), Type::CVoid));
                // 可变参数必须是最后一个参数
                if parser.check(&Token::Comma) {
                    return Err(parser.error("Varargs parameter must be the last parameter"));
                }
                break;
            }

            // 检查是否是可变参数类型（type...）
            let param_type = parse_type(parser)?;

            // 检查是否有 ... 标记
            let is_varargs = parser.match_token(&Token::DotDotDot);

            if is_varargs {
                // type... 形式的可变参数，需要一个名称
                let name = parser.consume_identifier("Expected parameter name")?;
                params.push(ParameterInfo::new_varargs(name, param_type));
                // 可变参数必须是最后一个参数
                if parser.match_token(&Token::Comma) {
                    return Err(parser.error("Varargs parameter must be the last parameter"));
                }
                break;
            } else {
                let name = parser.consume_identifier("Expected parameter name")?;
                params.push(ParameterInfo::new(name, param_type));
            }

            if !parser.match_token(&Token::Comma) {
                break;
            }
        }
    }

    Ok(params)
}