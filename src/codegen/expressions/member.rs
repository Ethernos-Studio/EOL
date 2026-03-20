//! 成员访问表达式代码生成
//!
//! 处理静态字段访问、对象成员访问和数组 length 属性。

use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::cayResult;

impl IRGenerator {
    /// 生成数组长度访问代码（用于 .length 属性或 .length() 方法）
    ///
    /// # Arguments
    /// * `array_expr` - 数组表达式
    pub fn generate_array_length_access(&mut self, array_expr: &Expr) -> cayResult<String> {
        let obj = self.generate_expression(array_expr)?;
        let (obj_type, obj_val) = self.parse_typed_value(&obj);

        // 首先将数组指针转换为 i8*
        let obj_i8 = self.new_temp();
        self.emit_line(&format!("  {} = bitcast {} {} to i8*", obj_i8, obj_type, obj_val));

        // 数组长度存储在数组指针前面的 8 字节中
        // 计算长度地址：array_ptr - 8
        let len_ptr_i8 = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr i8, i8* {}, i64 -8", len_ptr_i8, obj_i8));

        // 将长度指针转换为 i32*
        let len_ptr = self.new_temp();
        self.emit_line(&format!("  {} = bitcast i8* {} to i32*", len_ptr, len_ptr_i8));

        // 加载长度（作为 i32）
        let len_val = self.new_temp();
        self.emit_line(&format!("  {} = load i32, i32* {}, align 4", len_val, len_ptr));

        Ok(format!("i32 {}", len_val))
    }

    /// 生成成员访问表达式代码
    ///
    /// # Arguments
    /// * `member` - 成员访问表达式
    pub fn generate_member_access(&mut self, member: &MemberAccessExpr) -> cayResult<String> {
        // 检查是否是静态字段访问: ClassName.fieldName
        if let Expr::Identifier(class_name) = &*member.object {
            let static_key = format!("{}.{}", class_name, member.member);
            if let Some(field_info) = self.static_field_map.get(&static_key).cloned() {
                // 静态字段访问 - 返回全局变量的指针
                let temp = self.new_temp();
                self.emit_line(&format!("  {} = load {}, {}* {}, align {}",
                    temp, field_info.llvm_type, field_info.llvm_type, field_info.name,
                    self.get_type_align(&field_info.llvm_type)));
                return Ok(format!("{} {}", field_info.llvm_type, temp));
            }
        }

        // 特殊处理数组的 .length 属性
        if member.member == "length" {
            let obj = self.generate_expression(&member.object)?;
            let (obj_type, obj_val) = self.parse_typed_value(&obj);

            // 检查是否是数组类型（以 * 结尾）
            if obj_type.ends_with("*") {
                return self.generate_array_length_access(&member.object);
            }
        }
        
        // 处理实例字段访问: this.fieldName 或 obj.fieldName
        
        // 确定对象所属的类
        let class_name_opt: Option<String> = if let Expr::Identifier(name) = &*member.object {
            let name_str = name.as_ref();
            if name_str == "this" {
                Some(self.current_class.clone())
            } else {
                // 尝试从变量类型推断类名
                self.var_class_map.get(name_str).cloned()
            }
        } else {
            None
        };
        
        if let Some(class_name) = class_name_opt {
            if let Some(field_info) = self.get_instance_field(&class_name, &member.member).cloned() {
                // 实例字段访问
                
                // 获取对象指针
                // 对于 this，从作用域管理器获取 this_ptr 的 LLVM 名称；对于其他变量，加载其值
                let obj_ptr = if let Expr::Identifier(name) = &*member.object {
                    if name == "this" {
                        // 从作用域管理器获取 this_ptr 的 LLVM 名称，然后加载其值
                        let this_llvm_name = self.scope_manager.get_llvm_name("this_ptr")
                            .unwrap_or_else(|| "this_ptr_s1".to_string());
                        let temp = self.new_temp();
                        self.emit_line(&format!("  {} = load i8*, i8** %{}, align 8", 
                            temp, this_llvm_name));
                        temp
                    } else {
                        // 其他变量：生成表达式并提取值
                        let obj = self.generate_expression(&member.object)?;
                        let (_, obj_val) = self.parse_typed_value(&obj);
                        obj_val
                    }
                } else {
                    let obj = self.generate_expression(&member.object)?;
                    let (_, obj_val) = self.parse_typed_value(&obj);
                    obj_val
                };
                
                // 计算字段地址: obj_ptr + offset
                let field_ptr_i8 = self.new_temp();
                self.emit_line(&format!("  {} = getelementptr i8, i8* {}, i64 {}", 
                    field_ptr_i8, obj_ptr, field_info.offset));
                
                // 将字段指针转换为正确类型的指针
                let field_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to {}*", 
                    field_ptr, field_ptr_i8, field_info.llvm_type));
                
                // 加载字段值
                let field_val = self.new_temp();
                self.emit_line(&format!("  {} = load {}, {}* {}, align {}", 
                    field_val, field_info.llvm_type, field_info.llvm_type, field_ptr,
                    self.get_type_align(&field_info.llvm_type)));
                
                return Ok(format!("{} {}", field_info.llvm_type, field_val));
            }
        }
        
        // 目前仅支持将成员访问视为对象指针的占位符
        // 生成对象表达式并返回其指针值
        let obj = self.generate_expression(&member.object)?;
        let (_, obj_val) = self.parse_typed_value(&obj);
        Ok(format!("i8* {}", obj_val))
    }
}
