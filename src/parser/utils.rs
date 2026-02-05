//! 解析器辅助方法

use crate::lexer::{Token, TokenWithLocation};
use crate::error::{EolResult, EolError, parser_error, SourceLocation};
use super::Parser;

/// 检查是否到达令牌流末尾
pub fn is_at_end(parser: &Parser) -> bool {
    parser.pos >= parser.tokens.len() - 1
}

/// 获取当前令牌
pub fn current_token(parser: &Parser) -> &Token {
    &parser.tokens[parser.pos].token
}

/// 获取当前位置
pub fn current_loc(parser: &Parser) -> SourceLocation {
    parser.tokens[parser.pos].loc.clone()
}

/// 获取上一个位置
pub fn previous_loc(parser: &Parser) -> SourceLocation {
    if parser.pos > 0 {
        parser.tokens[parser.pos - 1].loc.clone()
    } else {
        parser.tokens[0].loc.clone()
    }
}

/// 前进到下一个令牌
pub fn advance(parser: &mut Parser) -> &Token {
    if !is_at_end(parser) {
        parser.pos += 1;
    }
    &parser.tokens[parser.pos - 1].token
}

/// 检查当前令牌是否匹配给定令牌
pub fn check(parser: &Parser, token: &Token) -> bool {
    if is_at_end(parser) {
        false
    } else {
        current_token(parser) == token
    }
}

/// 如果匹配则消耗令牌
pub fn match_token(parser: &mut Parser, token: &Token) -> bool {
    if check(parser, token) {
        advance(parser);
        true
    } else {
        false
    }
}

/// 消耗指定令牌，否则报错
pub fn consume<'a>(parser: &'a mut Parser, token: &Token, message: &str) -> EolResult<&'a Token> {
    if check(parser, token) {
        Ok(advance(parser))
    } else {
        // 如果期望分号但没找到，使用上一个token的位置
        let loc = if message.contains("';'") {
            previous_loc(parser)
        } else {
            current_loc(parser)
        };
        Err(parser_error(loc.line, loc.column, message))
    }
}

/// 消耗标识符
pub fn consume_identifier(parser: &mut Parser, message: &str) -> EolResult<String> {
    if let Token::Identifier(name) = current_token(parser) {
        let name = name.clone();
        advance(parser);
        Ok(name)
    } else {
        Err(error(parser, message))
    }
}

/// 创建错误
pub fn error(parser: &Parser, message: &str) -> EolError {
    let loc = &parser.tokens[parser.pos].loc;
    parser_error(loc.line, loc.column, message)
}