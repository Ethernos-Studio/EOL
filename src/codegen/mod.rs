//! cay LLVM IR 代码生成器
//!
//! 本模块将 cay AST 转换为 LLVM IR 代码。
//! 已重构为多个子模块以提高可维护性。

pub mod context;
mod types;
mod expressions;
mod statements;
pub mod runtime;
mod generator;
mod platform;
pub mod obfuscator;

// 公开 IRGenerator 作为代码生成器的入口
pub use context::IRGenerator;