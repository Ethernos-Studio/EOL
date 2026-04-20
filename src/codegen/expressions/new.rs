//! new 表达式代码生成
//!
//! 处理对象创建和数组创建。

use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::cayResult;

impl IRGenerator {
    /// 生成 new 表达式代码
    ///
    /// # Arguments
    /// * `new_expr` - new 表达式
    pub fn generate_new_expression(&mut self, new_expr: &NewExpr) -> cayResult<String> {
        let class_name = &new_expr.class_name;
        let type_id_value = self.get_type_id_value(class_name).unwrap_or(0);

        // 获取类布局信息，确定对象大小
        let obj_size = self.get_class_layout(class_name)
            .map(|layout| layout.total_size as i64)
            .unwrap_or(8i64); // 默认最小大小

        let calloc_temp = self.new_temp();
        self.emit_line(&format!("  {} = call i8* @calloc(i64 1, i64 {})", calloc_temp, obj_size));

        let type_id_ptr = self.new_temp();
        self.emit_line(&format!("  {} = bitcast i8* {} to i32*", type_id_ptr, calloc_temp));
        self.emit_line(&format!("  store i32 {}, i32* {}", type_id_value, type_id_ptr));

        // 调用构造函数（无论是否有参数）
        // 先推断参数类型
        let mut param_types = Vec::new();
        for arg in &new_expr.args {
            let arg_type = self.infer_argument_type(arg);
            param_types.push(arg_type);
        }
        
        // 生成参数值
        let mut arg_values = Vec::new();
        for arg in &new_expr.args {
            let arg_val = self.generate_expression(arg)?;
            arg_values.push(arg_val);
        }
        
        // 生成构造函数名（使用推断的参数类型）
        let ctor_name = self.generate_constructor_call_name_with_types(class_name, &param_types);
        
        // 生成参数列表
        let mut arg_strs = vec![format!("i8* {}", calloc_temp)];
        arg_strs.extend(arg_values);
        
        // 调用构造函数
        self.emit_line(&format!("  call void @{}({})",
            ctor_name, arg_strs.join(", ")));

        let cast_temp = self.new_temp();
        self.emit_line(&format!("  {} = bitcast i8* {} to i8*", cast_temp, calloc_temp));
        Ok(format!("i8* {}", cast_temp))
    }
    
    /// 推断参数类型（返回类型签名）
    fn infer_argument_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    LiteralValue::Int32(_) => "i".to_string(),
                    LiteralValue::Int64(_) => "l".to_string(),
                    LiteralValue::Float32(_) => "f".to_string(),
                    LiteralValue::Float64(_) => "d".to_string(),
                    LiteralValue::Bool(_) => "b".to_string(),
                    LiteralValue::Char(_) => "c".to_string(),
                    LiteralValue::String(_) => "s".to_string(),
                    LiteralValue::Null => "o".to_string(),
                }
            }
            Expr::Identifier(ident) => {
                // 查找变量类型
                if let Some(cay_type) = self.var_cay_types.get(&ident.name) {
                    self.type_to_signature(cay_type)
                } else {
                    "i".to_string() // 默认int
                }
            }
            Expr::MemberAccess(_) => {
                // 简化处理，默认int
                "i".to_string()
            }
            Expr::Binary(binary) => {
                // 二元表达式的类型通常是左操作数的类型
                self.infer_argument_type(&binary.left)
            }
            Expr::Unary(unary) => {
                self.infer_argument_type(&unary.operand)
            }
            Expr::Cast(cast) => {
                self.type_to_signature(&cast.target_type)
            }
            Expr::Call(_) => {
                // 简化处理，默认int
                "i".to_string()
            }
            _ => "i".to_string(), // 默认int
        }
    }
}
