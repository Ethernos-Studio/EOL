//! 语句分发代码生成
//!
//! 处理语句类型的分发。

use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::cayResult;

impl IRGenerator {
    /// 生成单个语句代码
    pub fn generate_statement(&mut self, stmt: &Stmt) -> cayResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expression(expr)?;
            }
            Stmt::VarDecl(var) => {
                self.generate_var_decl(var)?;
            }
            Stmt::Return(expr) => {
                self.generate_return_statement(expr)?;
            }
            Stmt::Block(block) => {
                // 检查是否是多变量声明生成的块（只包含 VarDecl）
                let is_multi_var_decl = block.statements.iter().all(|s| matches!(s, Stmt::VarDecl(_)));
                if is_multi_var_decl {
                    // 多变量声明不创建新作用域，在当前作用域内声明所有变量
                    for stmt in &block.statements {
                        if let Stmt::VarDecl(var) = stmt {
                            self.generate_var_decl(var)?;
                        }
                    }
                } else {
                    self.generate_block(block)?;
                }
            }
            Stmt::If(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }
            Stmt::While(while_stmt) => {
                self.generate_while_statement(while_stmt)?;
            }
            Stmt::For(for_stmt) => {
                self.generate_for_statement(for_stmt)?;
            }
            Stmt::DoWhile(do_while_stmt) => {
                self.generate_do_while_statement(do_while_stmt)?;
            }
            Stmt::Switch(switch_stmt) => {
                self.generate_switch_statement(switch_stmt)?;
            }
            Stmt::Scope(scope_stmt) => {
                self.generate_scope(scope_stmt)?;
            }
            Stmt::Break(label) => {
                self.generate_break_statement(label)?;
            }
            Stmt::Continue(label) => {
                self.generate_continue_statement(label)?;
            }
        }
        Ok(())
    }
}
