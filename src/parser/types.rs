//! 类型解析

use crate::types::Type;
use crate::error::cayResult;
use super::Parser;

/// 解析类型（支持多维数组和指针）
pub fn parse_type(parser: &mut Parser) -> cayResult<Type> {
    let base_type = match parser.current_token() {
        crate::lexer::Token::Int => { parser.advance(); Type::Int32 }
        crate::lexer::Token::Long => { parser.advance(); Type::Int64 }
        crate::lexer::Token::Float => { parser.advance(); Type::Float32 }
        crate::lexer::Token::Double => { parser.advance(); Type::Float64 }
        crate::lexer::Token::Bool => { parser.advance(); Type::Bool }
        crate::lexer::Token::String => { parser.advance(); Type::String }
        crate::lexer::Token::Char => { parser.advance(); Type::Char }
        // FFI 类型
        crate::lexer::Token::CInt => { parser.advance(); Type::CInt }
        crate::lexer::Token::CUInt => { parser.advance(); Type::CUInt }
        crate::lexer::Token::CLong => { parser.advance(); Type::CLong }
        crate::lexer::Token::CShort => { parser.advance(); Type::CShort }
        crate::lexer::Token::CUShort => { parser.advance(); Type::CUShort }
        crate::lexer::Token::CChar => { parser.advance(); Type::CChar }
        crate::lexer::Token::CUChar => { parser.advance(); Type::CUChar }
        crate::lexer::Token::CFloat => { parser.advance(); Type::CFloat }
        crate::lexer::Token::CDouble => { parser.advance(); Type::CDouble }
        crate::lexer::Token::SizeT => { parser.advance(); Type::SizeT }
        crate::lexer::Token::SSizeT => { parser.advance(); Type::SSizeT }
        crate::lexer::Token::UIntPtr => { parser.advance(); Type::UIntPtr }
        crate::lexer::Token::IntPtr => { parser.advance(); Type::IntPtr }
        crate::lexer::Token::CVoid => { parser.advance(); Type::CVoid }
        crate::lexer::Token::CBool => { parser.advance(); Type::CBool }
        crate::lexer::Token::Identifier(name) => {
            let name = name.clone();
            parser.advance();
            Type::Object(name)
        }
        _ => return Err(parser.error("Expected type")),
    };

    // 检查指针类型 Type*（支持多级指针 Type**）
    let mut result_type = base_type;
    while parser.match_token(&crate::lexer::Token::Star) {
        result_type = Type::Pointer(Box::new(result_type));
    }

    // 检查多维数组类型 Type[][]...
    while parser.match_token(&crate::lexer::Token::LBracket) {
        parser.consume(&crate::lexer::Token::RBracket, "Expected ']' after '['")?;
        result_type = Type::Array(Box::new(result_type));
    }

    Ok(result_type)
}

/// 检查当前token是否是类型token
pub fn is_type_token(parser: &Parser) -> bool {
    matches!(parser.current_token(),
        crate::lexer::Token::Int | crate::lexer::Token::Long | crate::lexer::Token::Float |
        crate::lexer::Token::Double | crate::lexer::Token::Bool | crate::lexer::Token::String |
        crate::lexer::Token::Char | crate::lexer::Token::Identifier(_) |
        // FFI 类型
        crate::lexer::Token::CInt | crate::lexer::Token::CUInt | crate::lexer::Token::CLong | 
        crate::lexer::Token::CShort | crate::lexer::Token::CUShort |
        crate::lexer::Token::CChar | crate::lexer::Token::CUChar |
        crate::lexer::Token::CFloat | crate::lexer::Token::CDouble |
        crate::lexer::Token::SizeT | crate::lexer::Token::SSizeT | crate::lexer::Token::UIntPtr |
        crate::lexer::Token::IntPtr | crate::lexer::Token::CVoid | crate::lexer::Token::CBool
    )
}

/// 检查当前token是否是原始类型token
pub fn is_primitive_type_token(parser: &Parser) -> bool {
    matches!(parser.current_token(),
        crate::lexer::Token::Int | crate::lexer::Token::Long | crate::lexer::Token::Float |
        crate::lexer::Token::Double | crate::lexer::Token::Bool | crate::lexer::Token::String |
        crate::lexer::Token::Char
    )
}