//! 类型检查实现

use crate::ast::*;
use crate::types::{Type, ParameterInfo};
use crate::error::{cayResult, semantic_error};
use super::analyzer::SemanticAnalyzer;
use super::symbol_table::SemanticSymbolInfo;

impl SemanticAnalyzer {
    /// 类型检查程序
    pub fn type_check_program(&mut self, program: &Program) -> cayResult<()> {
        for class in &program.classes {
            self.current_class = Some(class.name.clone());
            
            for member in &class.members {
                match member {
                    ClassMember::Method(method) => {
                        self.current_method = Some(method.name.clone());
                        self.symbol_table.enter_scope();
                        
                        // 添加参数到符号表
                        for param in &method.params {
                            self.symbol_table.declare(
                                param.name.clone(),
                                SemanticSymbolInfo {
                                    name: param.name.clone(),
                                    symbol_type: param.param_type.clone(),
                                    is_final: false,
                                    is_initialized: true,
                                }
                            );
                        }
                        
                        // 类型检查方法体
                        if let Some(body) = &method.body {
                            self.type_check_statement(&Stmt::Block(body.clone()), Some(&method.return_type))?;
                        }
                        
                        self.symbol_table.exit_scope();
                        self.current_method = None;
                    }
                    ClassMember::Field(_) => {
                        // 字段类型检查暂不实现
                    }
                    ClassMember::Constructor(ctor) => {
                        // 构造函数类型检查
                        self.symbol_table.enter_scope();
                        
                        // 添加 this 到符号表
                        self.symbol_table.declare(
                            "this".to_string(),
                            SemanticSymbolInfo {
                                name: "this".to_string(),
                                symbol_type: Type::Object(class.name.clone()),
                                is_final: true,
                                is_initialized: true,
                            }
                        );
                        
                        // 添加参数到符号表
                        for param in &ctor.params {
                            self.symbol_table.declare(
                                param.name.clone(),
                                SemanticSymbolInfo {
                                    name: param.name.clone(),
                                    symbol_type: param.param_type.clone(),
                                    is_final: false,
                                    is_initialized: true,
                                }
                            );
                        }
                        
                        // 类型检查构造函数体
                        self.type_check_statement(&Stmt::Block(ctor.body.clone()), Some(&Type::Void))?;
                        
                        self.symbol_table.exit_scope();
                    }
                    ClassMember::Destructor(dtor) => {
                        // 析构函数类型检查
                        self.symbol_table.enter_scope();
                        
                        // 添加 this 到符号表
                        self.symbol_table.declare(
                            "this".to_string(),
                            SemanticSymbolInfo {
                                name: "this".to_string(),
                                symbol_type: Type::Object(class.name.clone()),
                                is_final: true,
                                is_initialized: true,
                            }
                        );
                        
                        // 类型检查析构函数体
                        self.type_check_statement(&Stmt::Block(dtor.body.clone()), Some(&Type::Void))?;
                        
                        self.symbol_table.exit_scope();
                    }
                    ClassMember::InstanceInitializer(block) => {
                        // 实例初始化块类型检查
                        self.symbol_table.enter_scope();
                        self.type_check_statement(&Stmt::Block(block.clone()), Some(&Type::Void))?;
                        self.symbol_table.exit_scope();
                    }
                    ClassMember::StaticInitializer(block) => {
                        // 静态初始化块类型检查
                        self.symbol_table.enter_scope();
                        self.type_check_statement(&Stmt::Block(block.clone()), Some(&Type::Void))?;
                        self.symbol_table.exit_scope();
                    }
                }
            }
            
            self.current_class = None;
        }
        Ok(())
    }

    /// 类型检查语句
    pub fn type_check_statement(&mut self, stmt: &Stmt, expected_return: Option<&Type>) -> cayResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.infer_expr_type(expr)?;
            }
            Stmt::VarDecl(var) => {
                let mut var_type = var.var_type.clone();
                
                // 处理 auto 类型推断
                if var_type == Type::Auto {
                    if let Some(init) = &var.initializer {
                        var_type = self.infer_expr_type(init)?;
                    } else {
                        self.errors.push(format!(
                            "'auto' variable declaration requires an initializer at line {}",
                            var.loc.line
                        ));
                        var_type = Type::Int32; // 默认回退类型
                    }
                }
                
                if let Some(init) = &var.initializer {
                    let init_type = self.infer_expr_type(init)?;
                    if !self.types_compatible(&init_type, &var_type) {
                        self.errors.push(format!(
                            "Cannot assign {} to {} at line {}",
                            init_type, var_type, var.loc.line
                        ));
                    }
                }
                
                self.symbol_table.declare(
                    var.name.clone(),
                    SemanticSymbolInfo {
                        name: var.name.clone(),
                        symbol_type: var_type,
                        is_final: var.is_final,
                        is_initialized: var.initializer.is_some(),
                    }
                );
            }
            Stmt::Return(expr) => {
                let return_type = if let Some(e) = expr {
                    self.infer_expr_type(e)?
                } else {
                    Type::Void
                };
                
                if let Some(expected) = expected_return {
                    if !self.types_compatible(&return_type, expected) {
                        self.errors.push(format!(
                            "Return type mismatch: expected {}, got {}",
                            expected, return_type
                        ));
                    }
                }
            }
            Stmt::Block(block) => {
                self.symbol_table.enter_scope();
                for stmt in &block.statements {
                    self.type_check_statement(stmt, expected_return)?;
                }
                self.symbol_table.exit_scope();
            }
            _ => {}
        }
        
        Ok(())
    }
}
