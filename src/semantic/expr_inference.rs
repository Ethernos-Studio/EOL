//! 表达式类型推断

use crate::ast::*;
use crate::types::Type;
use crate::error::{cayResult, semantic_error};
use super::analyzer::SemanticAnalyzer;
use super::symbol_table::SemanticSymbolInfo;

impl SemanticAnalyzer {
    /// 推断表达式类型
    pub fn infer_expr_type(&mut self, expr: &Expr) -> cayResult<Type> {
        match expr {
            Expr::Literal(lit) => match lit {
                LiteralValue::Int32(_) => Ok(Type::Int32),
                LiteralValue::Int64(_) => Ok(Type::Int64),
                LiteralValue::Float32(_) => Ok(Type::Float32),
                LiteralValue::Float64(_) => Ok(Type::Float64),
                LiteralValue::String(_) => Ok(Type::String),
                LiteralValue::Bool(_) => Ok(Type::Bool),
                LiteralValue::Char(_) => Ok(Type::Char),
                LiteralValue::Null => Ok(Type::Object("Object".to_string())),
            }
            Expr::Identifier(ident) => {
                let name = &ident.name;
                let loc = &ident.loc;
                
                // 检查是否在静态上下文中访问 this
                if self.current_method_is_static && name == "this" {
                    return Err(crate::error::undefined_identifier_error(
                        loc.line, loc.column, name
                    ));
                }
                
                // 首先检查是否是当前类的字段（包括静态和非静态）
                if let Some(current_class_name) = &self.current_class {
                    if let Some(class_info) = self.type_registry.get_class(current_class_name) {
                        if let Some(field_info) = class_info.fields.get(name) {
                            if field_info.is_static {
                                return Ok(field_info.field_type.clone());
                            } else if self.current_method_is_static {
                                // 静态方法中不能访问非静态字段
                                return Err(crate::error::semantic_error(
                                    loc.line, loc.column,
                                    format!("non-static variable {} cannot be referenced from a static context", name)
                                ));
                            }
                            // 非静态方法中返回字段类型
                            return Ok(field_info.field_type.clone());
                        }
                    }
                }
                
                if let Some(info) = self.symbol_table.lookup(name) {
                    Ok(info.symbol_type.clone())
                } else if self.type_registry.class_exists(name) {
                    // 标识符是类名，返回类类型（用于静态成员访问）
                    Ok(Type::Object(name.clone()))
                } else {
                    Err(crate::error::undefined_identifier_error(
                        loc.line, loc.column, name
                    ))
                }
            }
            Expr::Binary(bin) => self.infer_binary_type(bin),
            Expr::Unary(unary) => self.infer_unary_type(unary),
            Expr::Call(call) => self.infer_call_type(call),
            Expr::MemberAccess(member) => self.infer_member_access_type(member),
            Expr::New(new_expr) => self.infer_new_type(new_expr),
            Expr::Assignment(assign) => self.infer_assignment_type(assign),
            Expr::Cast(cast) => self.infer_cast_type(cast),
            Expr::ArrayCreation(arr) => self.infer_array_creation_type(arr),
            Expr::ArrayInit(init) => self.infer_array_init_type(init),
            Expr::ArrayAccess(arr) => self.infer_array_access_type(arr),
            Expr::MethodRef(method_ref) => self.infer_method_ref_type(method_ref),
            Expr::Lambda(lambda) => self.infer_lambda_type(lambda),
            Expr::Ternary(ternary) => self.infer_ternary_type(ternary),
            Expr::InstanceOf(instanceof) => self.infer_instanceof_type(instanceof),
        }
    }

    /// 推断二元表达式类型
    fn infer_binary_type(&mut self, bin: &BinaryExpr) -> cayResult<Type> {
        let left_type = self.infer_expr_type(&bin.left)?;
        let right_type = self.infer_expr_type(&bin.right)?;
        
        match bin.op {
            BinaryOp::Add => {
                // 字符串连接：两个操作数都必须是字符串
                if left_type == Type::String && right_type == Type::String {
                    Ok(Type::String)
                }
                // 字符串 + char：允许，结果为字符串
                else if left_type == Type::String && right_type == Type::Char {
                    Ok(Type::String)
                }
                // char + 字符串：允许，结果为字符串
                else if left_type == Type::Char && right_type == Type::String {
                    Ok(Type::String)
                }
                // 数值加法：两个操作数都必须是基本数值类型
                else if left_type.is_primitive() && right_type.is_primitive() {
                    // 类型提升
                    Ok(self.promote_types(&left_type, &right_type))
                } else {
                    Err(semantic_error(
                        bin.loc.line,
                        bin.loc.column,
                        format!("Cannot add {} and {}: addition requires both operands to be numeric or both to be strings", left_type, right_type)
                    ))
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type.is_primitive() && right_type.is_primitive() {
                    // 检查除零和模零（仅当右操作数是字面量0时）
                    if matches!(bin.op, BinaryOp::Div | BinaryOp::Mod) {
                        if let Expr::Literal(LiteralValue::Int32(0)) = bin.right.as_ref() {
                            return Err(semantic_error(
                                bin.loc.line,
                                bin.loc.column,
                                "/ by zero".to_string()
                            ));
                        }
                        if let Expr::Literal(LiteralValue::Int64(0)) = bin.right.as_ref() {
                            return Err(semantic_error(
                                bin.loc.line,
                                bin.loc.column,
                                "/ by zero".to_string()
                            ));
                        }
                    }
                    // 类型提升
                    Ok(self.promote_types(&left_type, &right_type))
                } else {
                    Err(semantic_error(
                        bin.loc.line,
                        bin.loc.column,
                        format!("Cannot apply {:?} to {} and {}: operator requires numeric operands", bin.op, left_type, right_type)
                    ))
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_type == Type::Bool && right_type == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(semantic_error(
                        bin.loc.line,
                        bin.loc.column,
                        "Logical operators require boolean operands"
                    ))
                }
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.promote_integer_types(&left_type, &right_type))
                } else {
                    Err(semantic_error(
                        bin.loc.line,
                        bin.loc.column,
                        format!("Bitwise operator {:?} requires integer operands, got {} and {}",
                               bin.op, left_type, right_type)
                    ))
                }
            }
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::UnsignedShr => {
                if left_type.is_integer() && right_type.is_integer() {
                    // 移位运算符的结果类型与左操作数相同（经过整数提升）
                    Ok(self.promote_integer_types(&left_type, &right_type))
                } else {
                    Err(semantic_error(
                        bin.loc.line,
                        bin.loc.column,
                        format!("Shift operator {:?} requires integer operands, got {} and {}",
                               bin.op, left_type, right_type)
                    ))
                }
            }
            _ => Ok(left_type),
        }
    }

    /// 推断一元表达式类型
    fn infer_unary_type(&mut self, unary: &UnaryExpr) -> cayResult<Type> {
        let operand_type = self.infer_expr_type(&unary.operand)?;
        match unary.op {
            UnaryOp::Neg => Ok(operand_type),
            UnaryOp::Not => {
                if operand_type == Type::Bool {
                    Ok(Type::Bool)
                } else {
                    Err(semantic_error(
                        unary.loc.line,
                        unary.loc.column,
                        "Cannot apply '!' to non-boolean"
                    ))
                }
            }
            UnaryOp::BitNot => Ok(operand_type),
            _ => Ok(operand_type),
        }
    }

    /// 推断函数调用类型
    fn infer_call_type(&mut self, call: &CallExpr) -> cayResult<Type> {
        // 特殊处理内置函数
        if let Expr::Identifier(name) = call.callee.as_ref() {
            // 内置输入函数的类型推断
            match name.as_str() {
                "print" | "println" => return Ok(Type::Void),
                "readInt" => return Ok(Type::Int32),
                "readLong" => return Ok(Type::Int64),
                "readFloat" => return Ok(Type::Float32),
                "readDouble" => return Ok(Type::Float64),
                "readLine" => return Ok(Type::String),
                "readChar" => return Ok(Type::Char),
                "readBool" => return Ok(Type::Bool),
                _ => {}
            }

            // 检查是否是 extern 函数（全局函数）
            if let Some(ref prog) = self.program {
                for extern_decl in &prog.extern_declarations {
                    for extern_func in &extern_decl.functions {
                        if extern_func.name == name.as_ref() {
                            // 检查参数数量（不包括可变参数）
                            let fixed_param_count = extern_func.params.iter()
                                .filter(|p| !p.is_varargs)
                                .count();
                            let has_varargs = extern_func.params.iter().any(|p| p.is_varargs);
                            
                            if has_varargs {
                                // 可变参数函数：参数数量 >= 固定参数数量
                                if call.args.len() < fixed_param_count {
                                    return Err(semantic_error(call.loc.line, call.loc.column,
                                        format!("Function '{}' requires at least {} arguments, but got {}",
                                            name, fixed_param_count, call.args.len())));
                                }
                            } else {
                                // 非可变参数函数：参数数量必须匹配
                                if call.args.len() != extern_func.params.len() {
                                    return Err(semantic_error(call.loc.line, call.loc.column,
                                        format!("Function '{}' requires {} arguments, but got {}",
                                            name, extern_func.params.len(), call.args.len())));
                                }
                            }
                            
                            // 返回 extern 函数的返回类型
                            return Ok(extern_func.return_type.clone());
                        }
                    }
                }
            }

            // 尝试查找当前类的方法（无对象调用）- 支持方法重载
            if let Some(ref current_class) = self.current_class.clone() {
                // 先推断所有参数类型
                let mut arg_types = Vec::new();
                for arg in &call.args {
                    arg_types.push(self.infer_expr_type(arg)?);
                }

                // 使用参数类型查找匹配的方法
                if let Some(method_info) = self.type_registry.find_method(current_class, name.as_ref(), &arg_types) {
                    let return_type = method_info.return_type.clone();
                    let params = method_info.params.clone();
                    // 检查参数类型兼容性（支持可变参数）
                    if let Err(msg) = self.check_arguments_compatible(&call.args, &params, call.loc.line, call.loc.column) {
                        return Err(semantic_error(call.loc.line, call.loc.column, msg));
                    }

                    return Ok(return_type);
                }
            }
        }

        // 支持成员调用: obj.method(...) 或 ClassName.method()（静态方法）
        if let Expr::MemberAccess(member) = call.callee.as_ref() {
            // 推断对象类型
            let obj_type = self.infer_expr_type(&member.object)?;

            // 处理 String 类型方法调用
            if obj_type == Type::String {
                return self.infer_string_method_call(&member.member, &call.args, call.loc.line, call.loc.column);
            }

            // 检查是否是类名（静态方法调用）- 支持方法重载
            if let Expr::Identifier(class_name) = &*member.object {
                let class_name_str = class_name.as_ref().to_string();
                // 先推断所有参数类型
                let mut arg_types = Vec::new();
                for arg in &call.args {
                    arg_types.push(self.infer_expr_type(arg)?);
                }

                if let Some(class_info) = self.type_registry.get_class(&class_name_str) {
                    // 使用参数类型查找匹配的静态方法
                    if let Some(method_info) = class_info.find_method(&member.member, &arg_types) {
                        if method_info.is_static {
                            let return_type = method_info.return_type.clone();
                            let params = method_info.params.clone();
                            // 检查参数类型兼容性（支持可变参数）
                            if let Err(msg) = self.check_arguments_compatible(&call.args, &params, call.loc.line, call.loc.column) {
                                return Err(semantic_error(call.loc.line, call.loc.column, msg));
                            }

                            return Ok(return_type);
                        }
                    }
                }
            }

            // 处理数组类型的 length() 方法调用（作为 .length 属性的语法糖）
            if let Type::Array(_) = &obj_type {
                if member.member == "length" && call.args.is_empty() {
                    return Ok(Type::Int32);
                }
            }

            // 处理类实例方法调用 - 支持方法重载
            if let Type::Object(class_name) = obj_type {
                // 先推断所有参数类型
                let mut arg_types = Vec::new();
                for arg in &call.args {
                    arg_types.push(self.infer_expr_type(arg)?);
                }

                // 使用参数类型查找匹配的方法
                if let Some(method_info) = self.type_registry.find_method(&class_name, &member.member, &arg_types) {
                    let return_type = method_info.return_type.clone();
                    let params = method_info.params.clone();
                    // 检查参数类型兼容性（支持可变参数）
                    if let Err(msg) = self.check_arguments_compatible(&call.args, &params, call.loc.line, call.loc.column) {
                        return Err(semantic_error(call.loc.line, call.loc.column, msg));
                    }

                    return Ok(return_type);
                } else {
                    return Err(semantic_error(
                        call.loc.line,
                        call.loc.column,
                        format!("Unknown method '{}' for class {}", member.member, class_name)
                    ));
                }
            }
        }

        // 如果找不到任何合适的方法，报错
        // 尝试提供更详细的错误信息
        if let Expr::Identifier(name) = call.callee.as_ref() {
            if let Some(ref current_class) = self.current_class {
                // 检查是否存在同名方法（参数不匹配）
                if let Some(class_info) = self.type_registry.get_class(current_class) {
                    if class_info.methods.contains_key(name.as_ref()) {
                        return Err(semantic_error(
                            call.loc.line,
                            call.loc.column,
                            format!("Method '{}' in class '{}' cannot be applied to given types: argument mismatch", name, current_class)
                        ));
                    }
                }
            }
            return Err(semantic_error(
                call.loc.line,
                call.loc.column,
                format!("Cannot find method '{}'", name)
            ));
        }

        if let Expr::MemberAccess(member) = call.callee.as_ref() {
            if let Expr::Identifier(class_name) = &*member.object {
                return Err(semantic_error(
                    call.loc.line,
                    call.loc.column,
                    format!("Method '{}' in class '{}' cannot be applied to given types: argument mismatch", member.member, class_name)
                ));
            }
            if let Type::Object(class_name) = self.infer_expr_type(&member.object)? {
                return Err(semantic_error(
                    call.loc.line,
                    call.loc.column,
                    format!("Method '{}' in class '{}' cannot be applied to given types: argument mismatch", member.member, class_name)
                ));
            }
        }

        Err(semantic_error(
            call.loc.line,
            call.loc.column,
            "Cannot resolve method call".to_string()
        ))
    }

    /// 推断成员访问类型
    fn infer_member_access_type(&mut self, member: &MemberAccessExpr) -> cayResult<Type> {
        // 检查是否是静态字段访问: ClassName.fieldName
        if let Expr::Identifier(class_name) = &*member.object {
            if let Some(class_info) = self.type_registry.get_class(class_name.as_ref()) {
                if let Some(field_info) = class_info.fields.get(&member.member) {
                    if field_info.is_static {
                        // 检查私有字段访问权限
                        if !field_info.is_public {
                            if let Some(current_class) = &self.current_class {
                                if current_class != class_name.as_ref() {
                                    return Err(semantic_error(
                                        member.loc.line,
                                        member.loc.column,
                                        format!("{} has private access in {}", member.member, class_name)
                                    ));
                                }
                            } else {
                                return Err(semantic_error(
                                    member.loc.line,
                                    member.loc.column,
                                    format!("{} has private access in {}", member.member, class_name)
                                ));
                            }
                        }
                        return Ok(field_info.field_type.clone());
                    }
                }
            }
        }

        // 成员访问类型检查
        let obj_type = self.infer_expr_type(&member.object)?;

        // 特殊处理数组的 .length 属性
        if member.member == "length" {
            if let Type::Array(_) = obj_type {
                return Ok(Type::Int32);  // length 返回 int
            }
        }

        // 特殊处理 String 类型方法
        if obj_type == Type::String {
            match member.member.as_str() {
                "length" => return Ok(Type::Int32),
                _ => {}
            }
        }

        // 检查静态方法中是否访问非静态成员
        if self.current_method_is_static {
            // 检查是否是 this 访问
            if let Expr::Identifier(name) = &*member.object {
                if name == "this" {
                    return Err(semantic_error(
                        member.loc.line,
                        member.loc.column,
                        format!("non-static variable {} cannot be referenced from a static context", member.member)
                    ));
                }
            }
        }

        // 类成员访问
        if let Type::Object(class_name) = obj_type {
            if let Some(class_info) = self.type_registry.get_class(&class_name) {
                if let Some(field_info) = class_info.fields.get(&member.member) {
                    // 检查静态方法中是否访问非静态字段
                    if self.current_method_is_static && !field_info.is_static {
                        // 检查是否是当前类的实例字段
                        if let Some(current_class) = &self.current_class {
                            if current_class == &class_name {
                                return Err(semantic_error(
                                    member.loc.line,
                                    member.loc.column,
                                    format!("non-static variable {} cannot be referenced from a static context", member.member)
                                ));
                            }
                        }
                    }
                    
                    // 检查私有字段访问权限
                    if !field_info.is_public {
                        if let Some(current_class) = &self.current_class {
                            if current_class != &class_name {
                                return Err(semantic_error(
                                    member.loc.line,
                                    member.loc.column,
                                    format!("{} has private access in {}", member.member, class_name)
                                ));
                            }
                        } else {
                            return Err(semantic_error(
                                member.loc.line,
                                member.loc.column,
                                format!("{} has private access in {}", member.member, class_name)
                            ));
                        }
                    }
                    return Ok(field_info.field_type.clone());
                }
            }
            return Err(semantic_error(
                member.loc.line,
                member.loc.column,
                format!("Unknown member '{}' for class {}", member.member, class_name)
            ));
        }

        Err(semantic_error(
            member.loc.line,
            member.loc.column,
            format!("Cannot access member '{}' on type {}", member.member, obj_type)
        ))
    }

    /// 推断 new 表达式类型
    fn infer_new_type(&mut self, new_expr: &NewExpr) -> cayResult<Type> {
        if let Some(class_info) = self.type_registry.get_class(&new_expr.class_name) {
            // 检查是否是抽象类
            if class_info.is_abstract {
                return Err(semantic_error(
                    new_expr.loc.line,
                    new_expr.loc.column,
                    format!("Cannot instantiate abstract class '{}'", new_expr.class_name)
                ));
            }
            Ok(Type::Object(new_expr.class_name.clone()))
        } else {
            Err(semantic_error(
                new_expr.loc.line,
                new_expr.loc.column,
                format!("Unknown class: {}", new_expr.class_name)
            ))
        }
    }

    /// 推断赋值表达式类型
    fn infer_assignment_type(&mut self, assign: &AssignmentExpr) -> cayResult<Type> {
        // 检查是否是 final 变量重新赋值
        if let Expr::Identifier(name) = &assign.target.as_ref() {
            if let Some(info) = self.symbol_table.lookup(name.as_ref()) {
                if info.is_final {
                    return Err(semantic_error(
                        assign.loc.line,
                        assign.loc.column,
                        format!("Cannot assign a value to final variable '{}'", name)
                    ));
                }
            }
        }

        let target_type = self.infer_expr_type(&assign.target)?;
        let value_type = self.infer_expr_type(&assign.value)?;

        if self.types_compatible(&value_type, &target_type) {
            Ok(target_type)
        } else {
            Err(semantic_error(
                assign.loc.line,
                assign.loc.column,
                format!("Cannot assign {} to {}", value_type, target_type)
            ))
        }
    }

    /// 推断类型转换表达式类型
    ///
    /// 验证类型转换的合法性并返回目标类型。
    /// 支持的转换类型：
    /// - 数值类型之间的转换（int <-> float，精度可能损失）
    /// - 引用类型之间的转换（继承层次结构内）
    /// - char 与 int 之间的转换
    ///
    /// # Arguments
    /// * `cast` - 类型转换表达式
    ///
    /// # Returns
    /// 成功时返回目标类型，失败时返回语义错误
    ///
    /// # Type Conversion Rules
    /// 1. 相同类型：允许（无实际效果）
    /// 2. 数值类型之间：允许（可能精度损失）
    /// 3. 引用类型之间：仅当存在继承关系时允许
    /// 4. char <-> int：允许
    /// 5. 数组类型之间：仅当元素类型兼容时允许
    /// 6. 其他组合：非法转换
    fn infer_cast_type(&mut self, cast: &CastExpr) -> cayResult<Type> {
        let source_type = self.infer_expr_type(&cast.expr)?;
        let target_type = &cast.target_type;
        
        // 相同类型，无需转换
        if source_type == *target_type {
            return Ok(target_type.clone());
        }
        
        // 检查转换是否合法
        if self.is_valid_cast(&source_type, target_type) {
            Ok(target_type.clone())
        } else {
            Err(semantic_error(
                cast.loc.line,
                cast.loc.column,
                format!("Invalid cast from {} to {}", source_type, target_type)
            ))
        }
    }
    
    /// 检查类型转换是否合法
    ///
    /// # Arguments
    /// * `from` - 源类型
    /// * `to` - 目标类型
    ///
    /// # Returns
    /// 如果转换合法返回 true
    fn is_valid_cast(&self, from: &Type, to: &Type) -> bool {
        use crate::types::Type;

        match (from, to) {
            // 相同类型
            (a, b) if a == b => true,

            // 数值类型之间的转换（所有组合都允许，可能精度损失）
            (Type::Int32, Type::Int64) |
            (Type::Int32, Type::Float32) |
            (Type::Int32, Type::Float64) |
            (Type::Int64, Type::Int32) |
            (Type::Int64, Type::Float32) |
            (Type::Int64, Type::Float64) |
            (Type::Float32, Type::Int32) |
            (Type::Float32, Type::Int64) |
            (Type::Float32, Type::Float64) |
            (Type::Float64, Type::Int32) |
            (Type::Float64, Type::Int64) |
            (Type::Float64, Type::Float32) => true,

            // char 与数值类型之间的转换
            (Type::Char, Type::Int32) |
            (Type::Char, Type::Int64) |
            (Type::Int32, Type::Char) |
            (Type::Int64, Type::Char) => true,

            // 任何基本类型都可以转换为 string
            (Type::Int32, Type::String) |
            (Type::Int64, Type::String) |
            (Type::Float32, Type::String) |
            (Type::Float64, Type::String) |
            (Type::Char, Type::String) |
            (Type::Bool, Type::String) => true,

            // FFI 类型与基本类型之间的转换
            // c_int <-> int
            (Type::CInt, Type::Int32) | (Type::Int32, Type::CInt) => true,
            // c_long <-> long
            (Type::CLong, Type::Int64) | (Type::Int64, Type::CLong) => true,
            // c_short <-> int (16位到32位)
            (Type::CShort, Type::Int32) | (Type::Int32, Type::CShort) => true,
            // c_char <-> int (8位到32位)
            (Type::CChar, Type::Int32) | (Type::Int32, Type::CChar) => true,
            (Type::CChar, Type::Char) | (Type::Char, Type::CChar) => true,
            // c_float <-> float
            (Type::CFloat, Type::Float32) | (Type::Float32, Type::CFloat) => true,
            // c_double <-> double
            (Type::CDouble, Type::Float64) | (Type::Float64, Type::CDouble) => true,
            // size_t/ssize_t <-> long 和 int
            (Type::SizeT, Type::Int64) | (Type::Int64, Type::SizeT) => true,
            (Type::SizeT, Type::Int32) | (Type::Int32, Type::SizeT) => true,
            (Type::SSizeT, Type::Int64) | (Type::Int64, Type::SSizeT) => true,
            (Type::SSizeT, Type::Int32) | (Type::Int32, Type::SSizeT) => true,
            // uintptr_t/intptr_t <-> long 和 int
            (Type::UIntPtr, Type::Int64) | (Type::Int64, Type::UIntPtr) => true,
            (Type::UIntPtr, Type::Int32) | (Type::Int32, Type::UIntPtr) => true,
            (Type::IntPtr, Type::Int64) | (Type::Int64, Type::IntPtr) => true,
            (Type::IntPtr, Type::Int32) | (Type::Int32, Type::IntPtr) => true,
            // c_bool <-> bool 和 int
            (Type::CBool, Type::Bool) | (Type::Bool, Type::CBool) => true,
            (Type::CBool, Type::Int32) | (Type::Int32, Type::CBool) => true,

            // FFI 类型之间的转换
            (Type::CInt, Type::CLong) | (Type::CLong, Type::CInt) => true,
            (Type::CInt, Type::CShort) | (Type::CShort, Type::CInt) => true,
            (Type::CInt, Type::CChar) | (Type::CChar, Type::CInt) => true,
            (Type::CFloat, Type::CDouble) | (Type::CDouble, Type::CFloat) => true,
            (Type::SizeT, Type::UIntPtr) | (Type::UIntPtr, Type::SizeT) => true,
            (Type::SSizeT, Type::IntPtr) | (Type::IntPtr, Type::SSizeT) => true,

            // 引用类型之间的转换：需要继承关系
            (Type::Object(from_name), Type::Object(to_name)) => {
                // 检查是否存在继承关系（双向）
                self.is_related_type(from_name, to_name)
            }

            // 数组类型之间的转换：元素类型兼容
            (Type::Array(from_elem), Type::Array(to_elem)) => {
                self.is_valid_cast(from_elem, to_elem)
            }

            // null 可以转换为任何引用类型
            (Type::Object(obj_name), Type::Object(_)) if obj_name == "Object" => true,

            // 其他组合都不合法
            _ => false,
        }
    }
    
    /// 检查两个类型是否存在继承关系（双向）
    ///
    /// 用于类型转换检查，允许向上转型（子类->父类）和向下转型（父类->子类）
    fn is_related_type(&self, type_a: &str, type_b: &str) -> bool {
        // 相同类型
        if type_a == type_b {
            return true;
        }
        
        // 检查 type_a 是否是 type_b 的子类型
        if self.is_subtype_of_by_name(type_a, type_b) {
            return true;
        }
        
        // 检查 type_b 是否是 type_a 的子类型
        if self.is_subtype_of_by_name(type_b, type_a) {
            return true;
        }
        
        false
    }
    
    /// 通过类型名称检查子类型关系
    ///
    /// 辅助函数，用于检查一个类型是否是另一个类型的子类型
    fn is_subtype_of_by_name(&self, subtype: &str, supertype: &str) -> bool {
        // 相同类型
        if subtype == supertype {
            return true;
        }
        
        // 所有类都是 Object 的子类型
        if supertype == "Object" {
            return self.type_registry.class_exists(subtype)
                || subtype == "String"
                || subtype == "Function";
        }
        
        // 迭代遍历继承链
        let mut current = subtype.to_string();
        let mut visited = std::collections::HashSet::new();
        
        loop {
            // 防止循环继承导致的无限循环
            if !visited.insert(current.clone()) {
                return false;
            }
            
            if let Some(class_info) = self.type_registry.get_class(&current) {
                match &class_info.parent {
                    Some(parent) => {
                        if parent == supertype {
                            return true;
                        }
                        current = parent.clone();
                    }
                    None => return false,
                }
            } else {
                // 内置类型检查
                return (subtype == "String" || subtype == "Function") && supertype == "Object";
            }
        }
    }

    /// 推断数组创建表达式类型
    fn infer_array_creation_type(&mut self, arr: &ArrayCreationExpr) -> cayResult<Type> {
        // 数组创建: new Type[size] 或 new Type[size1][size2]...
        // 检查所有维度的大小
        for (i, size) in arr.sizes.iter().enumerate() {
            let size_type = self.infer_expr_type(size)?;
            if !size_type.is_integer() {
                return Err(semantic_error(
                    arr.loc.line,
                    arr.loc.column,
                    format!("Array size at dimension {} must be integer, got {}", i + 1, size_type)
                ));
            }
            // 检查负数数组大小（仅当大小是字面量或一元负号表达式时）
            // 支持直接负数字面量如 -5（被解析为 Unary(Neg, Literal(5))）
            if let Expr::Literal(LiteralValue::Int32(n)) = size {
                if *n < 0 {
                    return Err(semantic_error(
                        arr.loc.line,
                        arr.loc.column,
                        format!("Array size cannot be negative: {}", n)
                    ));
                }
            }
            if let Expr::Literal(LiteralValue::Int64(n)) = size {
                if *n < 0 {
                    return Err(semantic_error(
                        arr.loc.line,
                        arr.loc.column,
                        format!("Array size cannot be negative: {}", n)
                    ));
                }
            }
            // 检查一元负号表达式如 -5
            if let Expr::Unary(unary) = size {
                if let UnaryOp::Neg = unary.op {
                    if let Expr::Literal(LiteralValue::Int32(n)) = unary.operand.as_ref() {
                        return Err(semantic_error(
                            arr.loc.line,
                            arr.loc.column,
                            format!("Array size cannot be negative: -{}", n)
                        ));
                    }
                    if let Expr::Literal(LiteralValue::Int64(n)) = unary.operand.as_ref() {
                        return Err(semantic_error(
                            arr.loc.line,
                            arr.loc.column,
                            format!("Array size cannot be negative: -{}", n)
                        ));
                    }
                }
            }
        }
        Ok(Type::Array(Box::new(arr.element_type.clone())))
    }

    /// 推断数组初始化表达式类型
    fn infer_array_init_type(&mut self, init: &ArrayInitExpr) -> cayResult<Type> {
        // 数组初始化: {1, 2, 3}
        // 需要上下文来推断类型，这里返回一个占位符类型
        // 实际类型会在变量声明时根据声明类型确定
        if init.elements.is_empty() {
            return Err(semantic_error(
                init.loc.line,
                init.loc.column,
                "Cannot infer type of empty array initializer".to_string()
            ));
        }
        // 推断第一个元素的类型作为数组元素类型
        let elem_type = self.infer_expr_type(&init.elements[0])?;
        Ok(Type::Array(Box::new(elem_type)))
    }

    /// 推断数组访问表达式类型
    fn infer_array_access_type(&mut self, arr: &ArrayAccessExpr) -> cayResult<Type> {
        // 数组访问: arr[index]
        let array_type = self.infer_expr_type(&arr.array)?;
        let index_type = self.infer_expr_type(&arr.index)?;

        if !index_type.is_integer() {
            return Err(semantic_error(
                arr.loc.line,
                arr.loc.column,
                format!("Array index must be integer, got {}", index_type)
            ));
        }

        match array_type {
            Type::Array(element_type) => Ok(*element_type),
            _ => Err(semantic_error(
                arr.loc.line,
                arr.loc.column,
                format!("Cannot index non-array type {}", array_type)
            )),
        }
    }

    /// 推断方法引用表达式类型
    fn infer_method_ref_type(&mut self, method_ref: &MethodRefExpr) -> cayResult<Type> {
        // 方法引用: ClassName::methodName 或 obj::methodName
        // 返回函数类型，包含参数类型和返回类型信息
        
        if let Some(ref class_name) = method_ref.class_name {
            // 检查类是否存在
            if !self.type_registry.class_exists(class_name) {
                return Err(semantic_error(
                    method_ref.loc.line,
                    method_ref.loc.column,
                    format!("Unknown class: {}", class_name)
                ));
            }
            // 获取方法信息
            if let Some(class_info) = self.type_registry.get_class(class_name) {
                if let Some(methods) = class_info.methods.get(&method_ref.method_name) {
                    if let Some(method_info) = methods.first() {
                        // 构建函数类型
                        let param_types: Vec<Type> = method_info.params.iter()
                            .map(|p| p.param_type.clone())
                            .collect();
                        let return_type = Box::new(method_info.return_type.clone());
                        
                        return Ok(Type::Function(Box::new(crate::types::FunctionType {
                            params: param_types,
                            return_type,
                            is_static: method_info.is_static,
                        })));
                    }
                } else {
                    return Err(semantic_error(
                        method_ref.loc.line,
                        method_ref.loc.column,
                        format!("Unknown method '{}' for class {}", method_ref.method_name, class_name)
                    ));
                }
            }
        } else if let Some(object) = method_ref.object.as_ref() {
            // 实例方法引用: obj::methodName
            let obj_type = self.infer_expr_type(object)?;
            if let Type::Object(class_name) = obj_type {
                if let Some(class_info) = self.type_registry.get_class(&class_name) {
                    if let Some(methods) = class_info.methods.get(&method_ref.method_name) {
                        if let Some(method_info) = methods.first() {
                            let param_types: Vec<Type> = method_info.params.iter()
                                .map(|p| p.param_type.clone())
                                .collect();
                            let return_type = Box::new(method_info.return_type.clone());
                            
                            return Ok(Type::Function(Box::new(crate::types::FunctionType {
                                params: param_types,
                                return_type,
                                is_static: false,
                            })));
                        }
                    }
                }
            }
        }
        
        // 无法确定具体函数类型，返回通用 Function 类型
        Ok(Type::Object("Function".to_string()))
    }

    /// 推断 Lambda 表达式类型
    fn infer_lambda_type(&mut self, lambda: &LambdaExpr) -> cayResult<Type> {
        // Lambda 表达式: (params) -> { body }
        // 创建新的作用域
        self.symbol_table.enter_scope();

        // 添加 Lambda 参数到符号表
        let mut param_types = Vec::new();
        for param in &lambda.params {
            let param_type = param.param_type.clone().unwrap_or(Type::Int32);
            param_types.push(param_type.clone());
            self.symbol_table.declare(
                param.name.clone(),
                SemanticSymbolInfo {
                    name: param.name.clone(),
                    symbol_type: param_type,
                    is_final: false,
                    is_initialized: true,
                }
            );
        }

        // 推断 Lambda 体类型
        let return_type = match &lambda.body {
            LambdaBody::Expr(expr) => {
                let expr_type = self.infer_expr_type(expr)?;
                Box::new(expr_type)
            }
            LambdaBody::Block(block) => {
                // 分析块中的语句，查找 return 语句
                let mut inferred_return: Option<Type> = None;
                for stmt in &block.statements {
                    if let Stmt::Return(ret_expr_opt) = stmt {
                        if let Some(ret_expr) = ret_expr_opt {
                            let ret_type = self.infer_expr_type(ret_expr)?;
                            inferred_return = Some(ret_type);
                        } else {
                            inferred_return = Some(Type::Void);
                        }
                        break; // 使用第一个 return 语句的类型
                    }
                }
                Box::new(inferred_return.unwrap_or(Type::Void))
            }
        };

        self.symbol_table.exit_scope();

        // 返回完整的函数类型
        Ok(Type::Function(Box::new(crate::types::FunctionType {
            params: param_types,
            return_type,
            is_static: true,
        })))
    }

    /// 推断三元运算符表达式类型
    fn infer_ternary_type(&mut self, ternary: &TernaryExpr) -> cayResult<Type> {
        // 推断条件表达式类型
        let cond_type = self.infer_expr_type(&ternary.condition)?;

        // 条件必须是布尔类型
        if cond_type != Type::Bool {
            return Err(semantic_error(
                ternary.loc.line,
                ternary.loc.column,
                format!("Ternary operator condition must be boolean, got {}", cond_type)
            ));
        }

        // 推断两个分支的类型
        let true_type = self.infer_expr_type(&ternary.true_branch)?;
        let false_type = self.infer_expr_type(&ternary.false_branch)?;

        // 两个分支类型必须兼容
        if true_type == false_type {
            Ok(true_type)
        } else if Self::is_numeric_type_helper(&true_type) && Self::is_numeric_type_helper(&false_type) {
            // 数值类型进行类型提升
            Ok(self.promote_types(&true_type, &false_type))
        } else {
            Err(semantic_error(
                ternary.loc.line,
                ternary.loc.column,
                format!("Ternary operator branches must have compatible types, got {} and {}", true_type, false_type)
            ))
        }
    }

    /// 推断 instanceof 表达式类型
    fn infer_instanceof_type(&mut self, instanceof: &InstanceOfExpr) -> cayResult<Type> {
        // 检查表达式类型
        let expr_type = self.infer_expr_type(&instanceof.expr)?;

        // 检查目标类型是否存在（类或接口）
        match &instanceof.target_type {
            Type::Object(class_name) => {
                if !self.type_registry.class_exists(class_name) && !self.type_registry.interface_exists(class_name) {
                    return Err(semantic_error(
                        instanceof.loc.line,
                        instanceof.loc.column,
                        format!("Unknown type in instanceof: {}", class_name)
                    ));
                }
            }
            _ => {
                // instanceof 只能用于引用类型
                return Err(semantic_error(
                    instanceof.loc.line,
                    instanceof.loc.column,
                    format!("instanceof can only be used with reference types, got {}", instanceof.target_type)
                ));
            }
        }

        // instanceof 返回布尔类型
        Ok(Type::Bool)
    }

    /// 辅助方法：检查类型是否为数值类型
    fn is_numeric_type_helper(ty: &Type) -> bool {
        matches!(ty, Type::Int32 | Type::Int64 | Type::Float32 | Type::Float64 | Type::Char)
    }
}
